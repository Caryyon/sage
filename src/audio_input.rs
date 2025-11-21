use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::{Arc, Mutex};

/// Audio input engine that captures microphone audio and converts it to grid patterns
/// This is the inverse of sonification - it gives SAGE "hearing"
pub struct AudioInputEngine {
    _stream: Option<Stream>,
    latest_grid: Arc<Mutex<[[f64; 22]; 1024]>>,  // Latest audio-derived grid
    is_listening: Arc<Mutex<bool>>,
}

impl AudioInputEngine {
    /// Create a new audio input engine and start capturing from the default microphone
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;

        let config = device.default_input_config()?;

        let latest_grid = Arc::new(Mutex::new([[0.0; 22]; 1024]));
        let is_listening = Arc::new(Mutex::new(true));

        // Start audio capture stream
        let sample_format = config.sample_format();
        let config: StreamConfig = config.into();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => Self::build_stream_f32(&device, &config, latest_grid.clone(), is_listening.clone())?,
            cpal::SampleFormat::I16 => Self::build_stream_i16(&device, &config, latest_grid.clone(), is_listening.clone())?,
            cpal::SampleFormat::U16 => Self::build_stream_u16(&device, &config, latest_grid.clone(), is_listening.clone())?,
            _ => return Err("Unsupported sample format".into()),
        };

        stream.play()?;

        Ok(Self {
            _stream: Some(stream),
            latest_grid,
            is_listening,
        })
    }

    /// Build an audio input stream for f32 samples
    fn build_stream_f32(
        device: &Device,
        config: &StreamConfig,
        grid: Arc<Mutex<[[f64; 22]; 1024]>>,
        is_listening: Arc<Mutex<bool>>,
    ) -> Result<Stream, Box<dyn std::error::Error>> {
        let sample_rate = config.sample_rate.0 as f64;
        let channels = config.channels as usize;
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = buffer.clone();

        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !*is_listening.lock().unwrap() {
                    return;
                }
                let mut buf = buffer_clone.lock().unwrap();
                buf.extend_from_slice(data);

                if buf.len() >= 1024 * channels {
                    let samples: Vec<f32> = if channels == 1 {
                        buf.drain(..1024).collect()
                    } else {
                        buf.drain(..1024 * channels)
                            .collect::<Vec<f32>>()
                            .chunks(channels)
                            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                            .collect()
                    };
                    if let Some(audio_grid) = Self::audio_to_grid(&samples, sample_rate) {
                        *grid.lock().unwrap() = audio_grid;
                    }
                }
            },
            |err| eprintln!("Audio input error: {}", err),
            None,
        )?;
        Ok(stream)
    }

    /// Build an audio input stream for i16 samples
    fn build_stream_i16(
        device: &Device,
        config: &StreamConfig,
        grid: Arc<Mutex<[[f64; 22]; 1024]>>,
        is_listening: Arc<Mutex<bool>>,
    ) -> Result<Stream, Box<dyn std::error::Error>> {
        let sample_rate = config.sample_rate.0 as f64;
        let channels = config.channels as usize;
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = buffer.clone();

        let stream = device.build_input_stream(
            config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                if !*is_listening.lock().unwrap() {
                    return;
                }
                let mut buf = buffer_clone.lock().unwrap();
                for &sample in data {
                    buf.push(sample as f32 / i16::MAX as f32);
                }

                if buf.len() >= 1024 * channels {
                    let samples: Vec<f32> = if channels == 1 {
                        buf.drain(..1024).collect()
                    } else {
                        buf.drain(..1024 * channels)
                            .collect::<Vec<f32>>()
                            .chunks(channels)
                            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                            .collect()
                    };
                    if let Some(audio_grid) = Self::audio_to_grid(&samples, sample_rate) {
                        *grid.lock().unwrap() = audio_grid;
                    }
                }
            },
            |err| eprintln!("Audio input error: {}", err),
            None,
        )?;
        Ok(stream)
    }

    /// Build an audio input stream for u16 samples
    fn build_stream_u16(
        device: &Device,
        config: &StreamConfig,
        grid: Arc<Mutex<[[f64; 22]; 1024]>>,
        is_listening: Arc<Mutex<bool>>,
    ) -> Result<Stream, Box<dyn std::error::Error>> {
        let sample_rate = config.sample_rate.0 as f64;
        let channels = config.channels as usize;
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = buffer.clone();

        let stream = device.build_input_stream(
            config,
            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                if !*is_listening.lock().unwrap() {
                    return;
                }
                let mut buf = buffer_clone.lock().unwrap();
                for &sample in data {
                    // Convert u16 [0, 65535] to f32 [-1.0, 1.0]
                    buf.push((sample as f32 / u16::MAX as f32) * 2.0 - 1.0);
                }

                if buf.len() >= 1024 * channels {
                    let samples: Vec<f32> = if channels == 1 {
                        buf.drain(..1024).collect()
                    } else {
                        buf.drain(..1024 * channels)
                            .collect::<Vec<f32>>()
                            .chunks(channels)
                            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                            .collect()
                    };
                    if let Some(audio_grid) = Self::audio_to_grid(&samples, sample_rate) {
                        *grid.lock().unwrap() = audio_grid;
                    }
                }
            },
            |err| eprintln!("Audio input error: {}", err),
            None,
        )?;
        Ok(stream)
    }

    /// Convert audio samples to a 32x32 grid using FFT
    /// This is the inverse of sonification: Audio → Grid
    fn audio_to_grid(samples: &[f32], sample_rate: f64) -> Option<[[f64; 22]; 1024]> {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(samples.len());

        // Convert samples to complex numbers
        let mut buffer: Vec<Complex<f32>> = samples.iter()
            .map(|&s| Complex::new(s, 0.0))
            .collect();

        // Perform FFT
        fft.process(&mut buffer);

        // Convert frequency spectrum to grid
        let mut grid: [[f64; 22]; 1024] = [[0.0; 22]; 1024];

        // Map frequencies to grid positions (inverse of sonification)
        // Sonification: Y position (0-32) → frequency (200-2000 Hz)
        // Audio Input: frequency → Y position
        let min_freq = 200.0;
        let max_freq = 2000.0;
        let freq_range = max_freq - min_freq;

        // Process FFT bins
        let num_bins = buffer.len() / 2;  // Only use positive frequencies
        for (bin_idx, complex) in buffer.iter().take(num_bins).enumerate() {
            let magnitude = (complex.re * complex.re + complex.im * complex.im).sqrt() as f64;
            let frequency = (bin_idx as f64 * sample_rate) / buffer.len() as f64;

            // Skip frequencies outside our range
            if frequency < min_freq || frequency > max_freq {
                continue;
            }

            // Map frequency to Y position (0-31)
            let y = ((frequency - min_freq) / freq_range * 31.0) as usize;
            let y = y.min(31);

            // Distribute across X axis (spread energy)
            // Higher magnitude → more activation
            let normalized_mag = (magnitude / 100.0).min(1.0);

            for x in 0..32 {
                let i = y * 32 + x;
                let alpha = normalized_mag * 0.8;  // Scale to 0.0-0.8 range

                // Set alpha channel (main activation)
                grid[i][3] = (grid[i][3] + alpha).min(1.0);
            }
        }

        Some(grid)
    }

    /// Get the latest audio-derived grid
    pub fn get_grid(&self) -> [[f64; 22]; 1024] {
        *self.latest_grid.lock().unwrap()
    }

    /// Check if currently listening
    pub fn is_listening(&self) -> bool {
        *self.is_listening.lock().unwrap()
    }

    /// Enable/disable listening
    pub fn set_listening(&self, enabled: bool) {
        *self.is_listening.lock().unwrap() = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_to_grid() {
        // Generate a test tone at 440 Hz (A4)
        let sample_rate = 44100.0;
        let frequency = 440.0;
        let duration = 1024;

        let samples: Vec<f32> = (0..duration)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * std::f32::consts::PI * frequency as f32 * t).sin()
            })
            .collect();

        let grid = AudioInputEngine::audio_to_grid(&samples, sample_rate);
        assert!(grid.is_some());

        // Check that the grid has non-zero values
        let grid = grid.unwrap();
        let total: f64 = grid.iter().flatten().sum();
        assert!(total > 0.0);
    }
}
