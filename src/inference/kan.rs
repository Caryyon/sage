//! KAN (Kolmogorov-Arnold Network) implementation using B-splines
//!
//! Each edge in the network has its own learnable B-spline function,
//! replacing the weight*activation of traditional MLPs with a univariate
//! nonlinear function per edge.
//!
//! Architecture: for a KAN layer mapping n_in → n_out, there are
//! n_in * n_out B-spline functions, each with `grid_size + order` coefficients.
//! Output_j = Σ_i  spline_{i,j}(input_i)

use rand::Rng;
use super::nca_predictor::NCA_CHANNELS;

/// Perception size: 9 cells × NCA_CHANNELS
const PERCEPTION_SIZE: usize = 9 * NCA_CHANNELS;

// ---------------------------------------------------------------------------
// B-spline evaluation
// ---------------------------------------------------------------------------

/// Evaluate B-spline basis functions of given order at point t.
/// Uses the Cox-de Boor recursion.
/// Returns a vector of basis function values (length = knots.len() - order - 1)
fn bspline_basis(t: f64, knots: &[f64], order: usize) -> Vec<f64> {
    let n = knots.len() - order - 1;
    if n == 0 {
        return vec![];
    }

    // Order 0 (piecewise constant)
    let mut prev: Vec<f64> = (0..knots.len() - 1)
        .map(|i| {
            if knots[i] <= t && t < knots[i + 1] {
                1.0
            } else if (t - knots[knots.len() - 1]).abs() < 1e-15 && knots[i] < knots[i + 1] && (knots[i + 1] - knots[knots.len() - 1]).abs() < 1e-15 {
                1.0 // include right endpoint for last non-degenerate interval
            } else {
                0.0
            }
        })
        .collect();

    // Build up to desired order via recursion
    for p in 1..=order {
        let m = prev.len() - 1;
        let mut curr = Vec::with_capacity(m);
        for i in 0..m {
            let left = if (knots[i + p] - knots[i]).abs() > 1e-30 {
                (t - knots[i]) / (knots[i + p] - knots[i]) * prev[i]
            } else {
                0.0
            };
            let right = if (knots[i + p + 1] - knots[i + 1]).abs() > 1e-30 {
                (knots[i + p + 1] - t) / (knots[i + p + 1] - knots[i + 1]) * prev[i + 1]
            } else {
                0.0
            };
            curr.push(left + right);
        }
        prev = curr;
    }

    prev.truncate(n);
    while prev.len() < n {
        prev.push(0.0);
    }
    prev
}

/// Generate clamped uniform knot vector.
fn make_knots(grid_size: usize, order: usize, lo: f64, hi: f64) -> Vec<f64> {
    let mut knots = Vec::with_capacity(grid_size + 2 * order + 1);
    for _ in 0..order {
        knots.push(lo);
    }
    for i in 0..=grid_size {
        knots.push(lo + (hi - lo) * i as f64 / grid_size as f64);
    }
    for _ in 0..order {
        knots.push(hi);
    }
    knots
}

// ---------------------------------------------------------------------------
// KAN Layer
// ---------------------------------------------------------------------------

/// SiLU activation: x * sigmoid(x)
#[inline]
fn silu(x: f64) -> f64 {
    x / (1.0 + (-x).exp())
}

/// A single KAN layer: n_in → n_out with B-spline edges + residual.
#[derive(Clone)]
pub struct KanLayer {
    pub n_in: usize,
    pub n_out: usize,
    pub grid_size: usize,
    pub order: usize,
    pub n_coeffs: usize,
    /// Spline coefficients: [n_out][n_in][n_coeffs]
    pub coeffs: Vec<Vec<Vec<f64>>>,
    /// Residual weights: [n_out][n_in]
    pub res_weights: Vec<Vec<f64>>,
    /// Bias per output
    pub bias: Vec<f64>,
    knots: Vec<f64>,
    lo: f64,
    hi: f64,
}

impl KanLayer {
    pub fn new(n_in: usize, n_out: usize, grid_size: usize, order: usize) -> Self {
        let n_coeffs = grid_size + order;
        let lo = -2.0;
        let hi = 2.0;
        let knots = make_knots(grid_size, order, lo, hi);

        let mut rng = rand::thread_rng();
        let scale = (1.0 / (n_in as f64 * n_coeffs as f64)).sqrt();

        let coeffs: Vec<Vec<Vec<f64>>> = (0..n_out)
            .map(|_| {
                (0..n_in)
                    .map(|_| (0..n_coeffs).map(|_| rng.gen_range(-scale..scale)).collect())
                    .collect()
            })
            .collect();

        let res_scale = (1.0 / n_in as f64).sqrt();
        let res_weights: Vec<Vec<f64>> = (0..n_out)
            .map(|_| (0..n_in).map(|_| rng.gen_range(-res_scale..res_scale)).collect())
            .collect();

        let bias = vec![0.0; n_out];

        Self {
            n_in, n_out, grid_size, order, n_coeffs,
            coeffs, res_weights, bias, knots, lo, hi,
        }
    }

    pub fn param_count(&self) -> usize {
        self.n_out * self.n_in * self.n_coeffs + self.n_out * self.n_in + self.n_out
    }

    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        debug_assert_eq!(input.len(), self.n_in);
        let mut output = self.bias.clone();

        for j in 0..self.n_out {
            for i in 0..self.n_in {
                let x = input[i].clamp(self.lo, self.hi);
                let basis = bspline_basis(x, &self.knots, self.order);

                let mut spline_val = 0.0;
                let n = basis.len().min(self.n_coeffs);
                for k in 0..n {
                    spline_val += self.coeffs[j][i][k] * basis[k];
                }

                output[j] += spline_val + self.res_weights[j][i] * silu(x);
            }
        }

        output
    }

    pub fn to_vec(&self) -> Vec<f64> {
        let mut v = Vec::with_capacity(self.param_count());
        for j in 0..self.n_out {
            for i in 0..self.n_in {
                v.extend(&self.coeffs[j][i]);
            }
        }
        for j in 0..self.n_out {
            v.extend(&self.res_weights[j]);
        }
        v.extend(&self.bias);
        v
    }

    pub fn from_vec_at(params: &[f64], offset: usize, n_in: usize, n_out: usize, grid_size: usize, order: usize) -> (Self, usize) {
        let n_coeffs = grid_size + order;
        let lo = -2.0;
        let hi = 2.0;
        let knots = make_knots(grid_size, order, lo, hi);
        let mut idx = offset;

        let mut coeffs = Vec::with_capacity(n_out);
        for _ in 0..n_out {
            let mut row = Vec::with_capacity(n_in);
            for _ in 0..n_in {
                row.push(params[idx..idx + n_coeffs].to_vec());
                idx += n_coeffs;
            }
            coeffs.push(row);
        }

        let mut res_weights = Vec::with_capacity(n_out);
        for _ in 0..n_out {
            res_weights.push(params[idx..idx + n_in].to_vec());
            idx += n_in;
        }

        let bias = params[idx..idx + n_out].to_vec();
        idx += n_out;

        (Self {
            n_in, n_out, grid_size, order, n_coeffs,
            coeffs, res_weights, bias, knots, lo, hi,
        }, idx)
    }
}

// ---------------------------------------------------------------------------
// KAN Network
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct KanNetwork {
    pub layers: Vec<KanLayer>,
}

impl KanNetwork {
    pub fn new(widths: &[usize], grid_size: usize, order: usize) -> Self {
        let layers: Vec<KanLayer> = widths.windows(2)
            .map(|w| KanLayer::new(w[0], w[1], grid_size, order))
            .collect();
        Self { layers }
    }

    pub fn param_count(&self) -> usize {
        self.layers.iter().map(|l| l.param_count()).sum()
    }

    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut x = input.to_vec();
        for (i, layer) in self.layers.iter().enumerate() {
            x = layer.forward(&x);
            if i < self.layers.len() - 1 {
                // Layer norm between layers to keep values in spline domain
                let mean = x.iter().sum::<f64>() / x.len() as f64;
                let var = x.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / x.len() as f64;
                let std = (var + 1e-8).sqrt();
                for v in &mut x {
                    *v = (*v - mean) / std;
                }
            }
        }
        // Final tanh * 0.1 for residual NCA update
        for v in &mut x {
            *v = v.tanh() * 0.1;
        }
        x
    }

    pub fn to_vec(&self) -> Vec<f64> {
        let mut v = Vec::with_capacity(self.param_count());
        for layer in &self.layers {
            v.extend(layer.to_vec());
        }
        v
    }

    pub fn from_vec(params: &[f64], widths: &[usize], grid_size: usize, order: usize) -> Self {
        let mut layers = Vec::new();
        let mut offset = 0;
        for w in widths.windows(2) {
            let (layer, new_offset) = KanLayer::from_vec_at(params, offset, w[0], w[1], grid_size, order);
            layers.push(layer);
            offset = new_offset;
        }
        Self { layers }
    }
}

// ---------------------------------------------------------------------------
// KAN NCA Weights
// ---------------------------------------------------------------------------

/// KAN-based NCA weights — drop-in alternative to NcaWeights.
///
/// Architecture: 72 → 24 → 8, grid=5, order=3 (cubic B-splines)
/// Layer 1: 24*72*(5+3) + 24*72 + 24 = 13,824 + 1,728 + 24 = 15,576
/// Layer 2: 8*24*(5+3) + 8*24 + 8 = 1,536 + 192 + 8 = 1,736
/// Total: 17,312
///
/// For comparable MLP params (~5.2K), use smaller grid or narrower hidden.
/// But KAN's advantage is expressiveness per param, so we keep ~5K:
/// 72 → 8, grid=5, order=3 (single layer)
/// 8*72*8 + 8*72 + 8 = 4,608 + 576 + 8 = 5,192 ← matches MLP exactly!
///
/// Or 2-layer for more expressiveness at same budget:
/// 72 → 16 → 8, grid=3, order=3 (6 coeffs per edge)
/// Layer 1: 16*72*6 + 16*72 + 16 = 6,912 + 1,152 + 16 = 8,080
/// Layer 2: 8*16*6 + 8*16 + 8 = 768 + 128 + 8 = 904
/// Total: 8,984
///
/// Let's use single-layer to match param count exactly:
#[derive(Clone)]
pub struct KanNcaWeights {
    pub network: KanNetwork,
    pub widths: Vec<usize>,
    pub grid_size: usize,
    pub order: usize,
}

impl KanNcaWeights {
    /// Create with configurable architecture
    pub fn with_config(widths: Vec<usize>, grid_size: usize, order: usize) -> Self {
        let network = KanNetwork::new(&widths, grid_size, order);
        Self { network, widths, grid_size, order }
    }

    /// Default architecture — 2-layer KAN, roughly matching MLP param count
    /// 72 → 16 → 8, grid=3, order=3 → ~8,984 params (vs MLP's 5,192)
    /// Slightly more params but KAN should use them more efficiently
    pub fn random() -> Self {
        // 144 → 64 → 16, grid=6, order=3 → ~102,480 params (matches MLP's 107K)
        let widths = vec![PERCEPTION_SIZE, 64, NCA_CHANNELS];
        let grid_size = 6;
        let order = 3;
        Self::with_config(widths, grid_size, order)
    }

    pub fn param_count(&self) -> usize {
        self.network.param_count()
    }

    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        self.network.forward(input)
    }

    pub fn to_vec(&self) -> Vec<f64> {
        self.network.to_vec()
    }

    pub fn from_vec(params: &[f64]) -> Self {
        let widths = vec![PERCEPTION_SIZE, 64, NCA_CHANNELS];
        let grid_size = 6;
        let order = 3;
        let network = KanNetwork::from_vec(params, &widths, grid_size, order);
        Self { network, widths, grid_size, order }
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.to_vec();
        let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_le_bytes()).collect();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let data: Vec<f64> = bytes
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        Ok(Self::from_vec(&data))
    }
}

// ---------------------------------------------------------------------------
// CellBrain trait
// ---------------------------------------------------------------------------

/// Trait for NCA cell update rules. Both MLP and KAN implement this.
pub trait CellBrain: Clone + Send {
    fn param_count(&self) -> usize;
    fn forward(&self, input: &[f64]) -> Vec<f64>;
    fn to_vec(&self) -> Vec<f64>;
    fn from_vec(params: &[f64]) -> Self;
    fn name(&self) -> &str;
    fn architecture(&self) -> String;
}

impl CellBrain for KanNcaWeights {
    fn param_count(&self) -> usize { self.param_count() }
    fn forward(&self, input: &[f64]) -> Vec<f64> { self.forward(input) }
    fn to_vec(&self) -> Vec<f64> { self.to_vec() }
    fn from_vec(params: &[f64]) -> Self { KanNcaWeights::from_vec(params) }
    fn name(&self) -> &str { "KAN" }
    fn architecture(&self) -> String {
        format!("KAN: {:?} (grid={}, order={}, cubic B-splines)", self.widths, self.grid_size, self.order)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bspline_basis_partition_of_unity() {
        let knots = make_knots(5, 3, 0.0, 1.0);
        for t_i in 0..=20 {
            let t = t_i as f64 / 20.0;
            let basis = bspline_basis(t, &knots, 3);
            let sum: f64 = basis.iter().sum();
            assert!((sum - 1.0).abs() < 1e-10, "Basis sum at t={}: {}", t, sum);
        }
    }

    #[test]
    fn test_bspline_basis_nonnegative() {
        let knots = make_knots(6, 3, -2.0, 2.0);
        for t_i in 0..=40 {
            let t = -2.0 + 4.0 * t_i as f64 / 40.0;
            let basis = bspline_basis(t, &knots, 3);
            for &b in &basis {
                assert!(b >= -1e-15, "Negative basis value {} at t={}", b, t);
            }
        }
    }

    #[test]
    fn test_kan_layer_forward_shape() {
        let layer = KanLayer::new(10, 5, 6, 3);
        let input = vec![0.5; 10];
        let output = layer.forward(&input);
        assert_eq!(output.len(), 5);
    }

    #[test]
    fn test_kan_layer_param_count() {
        let layer = KanLayer::new(10, 5, 6, 3);
        // coeffs: 5*10*9=450, res: 5*10=50, bias: 5 → 505
        assert_eq!(layer.param_count(), 505);
        assert_eq!(layer.to_vec().len(), 505);
    }

    #[test]
    fn test_kan_network_roundtrip() {
        let net = KanNetwork::new(&[10, 5, 3], 6, 3);
        let v = net.to_vec();
        let net2 = KanNetwork::from_vec(&v, &[10, 5, 3], 6, 3);
        let v2 = net2.to_vec();
        assert_eq!(v.len(), v2.len());
        for (a, b) in v.iter().zip(v2.iter()) {
            assert!((a - b).abs() < 1e-15);
        }
    }

    #[test]
    fn test_kan_nca_weights_roundtrip() {
        let w = KanNcaWeights::random();
        let v = w.to_vec();
        assert_eq!(v.len(), w.param_count());
        let w2 = KanNcaWeights::from_vec(&v);
        let v2 = w2.to_vec();
        assert_eq!(v.len(), v2.len());
        for (a, b) in v.iter().zip(v2.iter()) {
            assert!((a - b).abs() < 1e-15);
        }
    }

    #[test]
    fn test_kan_nca_forward() {
        let w = KanNcaWeights::random();
        let input = vec![0.0; PERCEPTION_SIZE];
        let output = w.forward(&input);
        assert_eq!(output.len(), NCA_CHANNELS);
        for &v in &output {
            assert!(v.abs() <= 0.1 + 1e-10);
        }
    }

    #[test]
    fn test_kan_nca_param_count() {
        let w = KanNcaWeights::random();
        eprintln!("KAN NCA param count: {}", w.param_count());
        // Should be in reasonable range
        assert!(w.param_count() > 1000, "Too few params: {}", w.param_count());
    }
}
