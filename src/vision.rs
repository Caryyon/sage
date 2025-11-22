use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use image::{RgbaImage, DynamicImage};
use std::sync::{Arc, Mutex};
use crate::grid::Grid;

/// SAGE Vision System - Gives SAGE the ability to see and perceive the visual world
pub struct SageVision {
    camera: Arc<Mutex<Camera>>,
    last_frame: Arc<Mutex<Option<RgbaImage>>>,
    grid_size: u32,
}

impl SageVision {
    /// Create a new SAGE vision system
    ///
    /// # Arguments
    /// * `camera_index` - Camera index (usually 0 for built-in webcam)
    /// * `grid_size` - Size to resize frames to (32 for NCA grid)
    pub fn new(camera_index: u32, grid_size: u32) -> Result<Self, String> {
        // Request camera with 640x480 resolution
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        let camera = Camera::new(CameraIndex::Index(camera_index), requested)
            .map_err(|e| format!("Failed to open camera: {}", e))?;

        Ok(Self {
            camera: Arc::new(Mutex::new(camera)),
            last_frame: Arc::new(Mutex::new(None)),
            grid_size,
        })
    }

    /// Open the camera stream
    pub fn open(&self) -> Result<(), String> {
        let mut camera = self.camera.lock().unwrap();
        camera.open_stream()
            .map_err(|e| format!("Failed to open camera stream: {}", e))?;
        Ok(())
    }

    /// Capture a frame from the camera
    pub fn capture_frame(&self) -> Result<RgbaImage, String> {
        let mut camera = self.camera.lock().unwrap();

        // Capture frame
        let frame = camera.frame()
            .map_err(|e| format!("Failed to capture frame: {}", e))?;

        // Decode to image
        let decoded = frame.decode_image::<RgbFormat>()
            .map_err(|e| format!("Failed to decode frame: {}", e))?;

        // Convert to DynamicImage
        let dynamic_img = DynamicImage::ImageRgb8(decoded);

        // Convert to RGBA
        let rgba_img = dynamic_img.to_rgba8();

        // Resize to grid size (32×32 for NCA)
        let resized = image::imageops::resize(
            &rgba_img,
            self.grid_size,
            self.grid_size,
            image::imageops::FilterType::Lanczos3
        );

        // Store last frame
        *self.last_frame.lock().unwrap() = Some(resized.clone());

        Ok(resized)
    }

    /// Get the last captured frame
    pub fn last_frame(&self) -> Option<RgbaImage> {
        self.last_frame.lock().unwrap().clone()
    }

    /// Convert an RGBA image to NCA grid format (Vec<Vec<Vec<f64>>>)
    /// Returns grid[y][x][channel] where channels are [R, G, B, A]
    pub fn image_to_nca_grid(&self, img: &RgbaImage) -> Vec<Vec<Vec<f64>>> {
        let height = img.height() as usize;
        let width = img.width() as usize;

        let mut grid = vec![vec![vec![0.0; 4]; width]; height];

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x as u32, y as u32);
                grid[y][x][0] = pixel[0] as f64 / 255.0;  // R
                grid[y][x][1] = pixel[1] as f64 / 255.0;  // G
                grid[y][x][2] = pixel[2] as f64 / 255.0;  // B
                grid[y][x][3] = pixel[3] as f64 / 255.0;  // A (always 1.0 for camera)
            }
        }

        grid
    }

    /// Convert an RGBA image to a Grid target for NCA training
    /// This is the key method for visual learning - SAGE learns to recreate what it sees
    pub fn frame_to_grid(&self, img: &RgbaImage) -> Grid {
        let size = self.grid_size as usize;
        let mut grid = Grid::new(size, size);

        // Resize image if needed (should already be grid_size x grid_size from capture)
        let resized = if img.width() != self.grid_size || img.height() != self.grid_size {
            image::imageops::resize(img, self.grid_size, self.grid_size, image::imageops::FilterType::Lanczos3)
        } else {
            img.clone()
        };

        for y in 0..size {
            for x in 0..size {
                let pixel = resized.get_pixel(x as u32, y as u32);

                // Set RGBA channels (0-3)
                grid.cells[y][x][0] = pixel[0] as f64 / 255.0;  // R
                grid.cells[y][x][1] = pixel[1] as f64 / 255.0;  // G
                grid.cells[y][x][2] = pixel[2] as f64 / 255.0;  // B
                grid.cells[y][x][3] = pixel[3] as f64 / 255.0;  // A (1.0 = alive)

                // Hidden channels (4-15) initialized to 0
                // Pattern condition channels (16-19) left at 0 for visual learning
                // Environmental channels (20-21) left at 0
            }
        }

        grid
    }

    /// Capture a frame and convert directly to Grid target
    /// Convenience method for visual learning pipeline
    pub fn capture_as_grid(&self) -> Result<Grid, String> {
        let frame = self.capture_frame()?;
        Ok(self.frame_to_grid(&frame))
    }

    /// Extract visual features from an image for concept association
    pub fn extract_features(&self, img: &RgbaImage) -> VisualFeatures {
        let mut brightness_sum = 0.0;
        let mut r_sum = 0.0;
        let mut g_sum = 0.0;
        let mut b_sum = 0.0;
        let pixel_count = (img.width() * img.height()) as f64;

        // Calculate average color and brightness
        for pixel in img.pixels() {
            let r = pixel[0] as f64 / 255.0;
            let g = pixel[1] as f64 / 255.0;
            let b = pixel[2] as f64 / 255.0;

            r_sum += r;
            g_sum += g;
            b_sum += b;
            brightness_sum += (r + g + b) / 3.0;
        }

        let avg_brightness = brightness_sum / pixel_count;
        let avg_r = r_sum / pixel_count;
        let avg_g = g_sum / pixel_count;
        let avg_b = b_sum / pixel_count;

        // Calculate color variance (how colorful vs grayscale)
        let color_variance = ((avg_r - avg_brightness).powi(2) +
                             (avg_g - avg_brightness).powi(2) +
                             (avg_b - avg_brightness).powi(2)).sqrt();

        // Dominant color
        let dominant_color = if avg_r > avg_g && avg_r > avg_b {
            "red"
        } else if avg_g > avg_r && avg_g > avg_b {
            "green"
        } else if avg_b > avg_r && avg_b > avg_g {
            "blue"
        } else {
            "neutral"
        };

        // Calculate edge strength (simple gradient)
        let mut edge_strength = 0.0;
        for y in 1..(img.height() - 1) {
            for x in 1..(img.width() - 1) {
                let center = img.get_pixel(x, y);
                let right = img.get_pixel(x + 1, y);
                let down = img.get_pixel(x, y + 1);

                let dx = (right[0] as i32 - center[0] as i32).abs() as f64;
                let dy = (down[0] as i32 - center[0] as i32).abs() as f64;
                edge_strength += (dx * dx + dy * dy).sqrt();
            }
        }
        edge_strength /= pixel_count;

        VisualFeatures {
            avg_brightness,
            avg_r,
            avg_g,
            avg_b,
            color_variance,
            dominant_color: dominant_color.to_string(),
            edge_strength,
        }
    }

    /// Stop the camera stream
    pub fn close(&self) -> Result<(), String> {
        let mut camera = self.camera.lock().unwrap();
        camera.stop_stream()
            .map_err(|e| format!("Failed to stop camera: {}", e))?;
        Ok(())
    }
}

/// Visual features extracted from an image
#[derive(Debug, Clone)]
pub struct VisualFeatures {
    pub avg_brightness: f64,
    pub avg_r: f64,
    pub avg_g: f64,
    pub avg_b: f64,
    pub color_variance: f64,
    pub dominant_color: String,
    pub edge_strength: f64,
}

impl VisualFeatures {
    /// Describe the visual features in natural language
    pub fn describe(&self) -> String {
        let brightness_desc = if self.avg_brightness > 0.7 {
            "bright"
        } else if self.avg_brightness > 0.3 {
            "moderately lit"
        } else {
            "dim"
        };

        let colorfulness = if self.color_variance > 0.2 {
            "very colorful"
        } else if self.color_variance > 0.1 {
            "somewhat colorful"
        } else {
            "mostly grayscale"
        };

        let detail = if self.edge_strength > 50.0 {
            "highly detailed with sharp edges"
        } else if self.edge_strength > 20.0 {
            "moderately detailed"
        } else {
            "smooth with soft edges"
        };

        format!(
            "I perceive a {} scene with a {} tone, {}. Dominant color: {}.",
            brightness_desc,
            colorfulness,
            detail,
            self.dominant_color
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_features_describe() {
        let features = VisualFeatures {
            avg_brightness: 0.8,
            avg_r: 0.9,
            avg_g: 0.7,
            avg_b: 0.6,
            color_variance: 0.15,
            dominant_color: "red".to_string(),
            edge_strength: 35.0,
        };

        let description = features.describe();
        assert!(description.contains("bright"));
        assert!(description.contains("red"));
    }
}
