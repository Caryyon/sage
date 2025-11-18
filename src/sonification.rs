use rodio::{OutputStream, Sink};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Sonification engine that converts SAGE's neural patterns into sound
pub struct SonificationEngine {
    _stream: OutputStream,
    sink: Arc<Mutex<Sink>>,
    _active: Arc<Mutex<bool>>,
}

impl SonificationEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(Self {
            _stream,
            sink: Arc::new(Mutex::new(sink)),
            _active: Arc::new(Mutex::new(false)),
        })
    }

    /// Convert a 32x32 grid into flowing arpeggiated harmonies
    /// Creates evolving sequences with consonant intervals that change each frame
    pub fn sonify_grid(&self, grid: &[[f64; 22]; 1024], duration_ms: u64) {
        let sample_rate = 44100;
        let duration_samples = (sample_rate as f64 * duration_ms as f64 / 1000.0) as usize;

        // Extract living cells and sort by Y position (lowest = bass)
        let mut active_cells = Vec::new();
        for (i, cell) in grid.iter().enumerate() {
            let alpha = cell[3];
            if alpha > 0.1 {
                let x = (i % 32) as f64 / 32.0;
                let y = (i / 32) as f64 / 32.0;
                active_cells.push((x, y, alpha));
            }
        }

        if active_cells.is_empty() {
            return;
        }

        // Sort by Y position to identify bass note
        active_cells.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut samples = vec![0.0f32; duration_samples * 2];

        // Bass drone: use lowest Y position as root frequency (600-800 Hz)
        let bass_y = active_cells[0].1;
        let root_freq = 600.0 + (bass_y * 200.0);

        // Arpeggiate through cells instead of playing all at once
        let num_cells = active_cells.len().min(8); // Use up to 8 cells
        let note_duration = duration_samples / num_cells.max(1);

        for (idx, (x, y, alpha)) in active_cells.iter().take(num_cells).enumerate() {
            // Determine harmonic relationship
            let freq = if idx == 0 {
                root_freq  // Bass: root note
            } else if idx % 4 == 1 {
                root_freq * 1.5  // Perfect 5th
            } else if idx % 4 == 2 {
                root_freq * 1.25  // Major 3rd
            } else if idx % 4 == 3 {
                root_freq * 2.0  // Octave
            } else {
                root_freq * (1.0 + (*y * 0.5))
            };

            // Each note plays for a portion of the total duration
            let start_sample = idx * note_duration;
            let end_sample = ((idx + 1) * note_duration).min(duration_samples);
            let note_samples = end_sample - start_sample;

            // Vary amplitude based on alpha (creates dynamic expression)
            let amplitude = (*alpha * 0.07) as f32;

            // Stereo spread based on X position
            let pan = *x as f32;
            let left_pan = ((1.0 - pan) * std::f32::consts::FRAC_PI_2).cos();
            let right_pan = (pan * std::f32::consts::FRAC_PI_2).cos();

            // Fast attack, longer decay for plucky feel
            let attack_samples = (note_samples / 10).max(100);  // 10% attack
            let decay_samples = (note_samples * 3 / 4).max(300); // 75% decay

            // Generate note
            for i in 0..note_samples {
                let global_i = start_sample + i;
                if global_i >= duration_samples {
                    break;
                }

                let t = i as f32 / sample_rate as f32;

                // Plucky envelope: quick attack, exponential decay
                let envelope = if i < attack_samples {
                    let progress = i as f32 / attack_samples as f32;
                    progress * progress  // Quadratic fade-in
                } else {
                    let decay_progress = (i - attack_samples) as f32 / decay_samples as f32;
                    (-decay_progress * 4.0).exp()  // Exponential decay
                };

                // Mix sine and triangle for warmer tone
                let sine = (2.0 * std::f32::consts::PI * freq as f32 * t).sin();
                let triangle = (4.0 / std::f32::consts::PI) *
                    ((2.0 * std::f32::consts::PI * freq as f32 * t).sin() +
                     (2.0 * std::f32::consts::PI * freq as f32 * 3.0 * t).sin() / 9.0);

                // Blend based on alpha (high alpha = more triangle/brightness)
                let blend = *alpha as f32;
                let wave = sine * (1.0 - blend * 0.3) + triangle * (blend * 0.3);

                // Subtle vibrato
                let vibrato = 1.0 + 0.002 * (2.0 * std::f32::consts::PI * 4.0 * t).sin();
                let osc = wave * vibrato;

                let sample = osc * amplitude * envelope;

                samples[global_i * 2] += sample * left_pan;
                samples[global_i * 2 + 1] += sample * right_pan;
            }
        }

        // Multi-tap reverb for spaciousness
        let reverb_samples = samples.clone();
        let delays = [
            (sample_rate as f32 * 0.03) as usize,  // 30ms
            (sample_rate as f32 * 0.05) as usize,  // 50ms
            (sample_rate as f32 * 0.08) as usize,  // 80ms
        ];
        for (tap_idx, &delay) in delays.iter().enumerate() {
            let decay = 0.12 / (tap_idx + 1) as f32;  // Each tap quieter
            for i in delay..samples.len() {
                samples[i] += reverb_samples[i - delay] * decay;
            }
        }

        // Gentle low-pass filter
        let alpha_lpf = 0.25f32;
        for i in 1..samples.len() {
            samples[i] = samples[i] * alpha_lpf + samples[i - 1] * (1.0 - alpha_lpf);
        }

        // Soft limiting
        let max_sample = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        if max_sample > 0.7 {
            for sample in &mut samples {
                let normalized = *sample / max_sample;
                let abs_norm = normalized.abs();
                let sign = if normalized >= 0.0 { 1.0 } else { -1.0 };
                *sample = if abs_norm < 0.66 {
                    normalized * 0.6
                } else {
                    sign * (0.66 - (1.0 - abs_norm).powi(3)) * 0.6
                };
            }
        }

        // Play the generated audio
        let source = rodio::buffer::SamplesBuffer::new(2, sample_rate, samples);
        if let Ok(sink) = self.sink.lock() {
            sink.append(source);
        }
    }

    /// Create an evolving soundscape that changes as the pattern develops
    pub fn sonify_evolution(&self, grids: &[[[f64; 22]; 1024]], frame_duration_ms: u64) {
        for grid in grids {
            self.sonify_grid(grid, frame_duration_ms);
            thread::sleep(Duration::from_millis(frame_duration_ms));
        }
    }

    /// Stop all audio playback
    pub fn stop(&self) {
        if let Ok(sink) = self.sink.lock() {
            sink.stop();
        }
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        self.sink.lock()
            .map(|sink| !sink.empty())
            .unwrap_or(false)
    }

    /// Set the volume (0.0 - 1.0)
    pub fn set_volume(&self, volume: f32) {
        if let Ok(sink) = self.sink.lock() {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }
}

/// Helper function to generate a simple tone
pub fn generate_tone(frequency: f32, duration_ms: u64, amplitude: f32) -> Vec<f32> {
    let sample_rate = 44100;
    let num_samples = (sample_rate as f64 * duration_ms as f64 / 1000.0) as usize;

    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tone_generation() {
        let tone = generate_tone(440.0, 100, 0.5);
        assert!(!tone.is_empty());
        assert!(tone.iter().all(|&s| s.abs() <= 0.5));
    }
}
