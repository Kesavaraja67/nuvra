// nuvra-mod-media/src/visualizer.rs
// 32-band audio FFT visualizer engine using rustfft (Section 14.2, D-PERF-01).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::Arc;

pub const NUM_BANDS: usize = 32;
pub const FFT_SIZE: usize = 1024;

pub struct AudioVisualizer {
    fft: Arc<dyn rustfft::Fft<f32>>,
    smoothing_decay: f32,
    prev_bands: [f32; NUM_BANDS],
}

impl AudioVisualizer {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        Self {
            fft,
            smoothing_decay: 0.85,
            prev_bands: [0.0; NUM_BANDS],
        }
    }

    /// Process PCM buffer into 32 normalized frequency bands [0.0, 1.0].
    pub fn process_samples(&mut self, samples: &[f32]) -> [f32; NUM_BANDS] {
        if samples.len() < FFT_SIZE {
            // Apply decay on starvation
            for band in &mut self.prev_bands {
                *band *= self.smoothing_decay;
            }
            return self.prev_bands;
        }

        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(FFT_SIZE)
            .enumerate()
            .map(|(i, &sample)| {
                // Apply Hann window function to minimize spectral leakage
                let window =
                    0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / FFT_SIZE as f32).cos());
                Complex::new(sample * window, 0.0)
            })
            .collect();

        self.fft.process(&mut buffer);

        // Group into 32 log-spaced perceptual bands
        let mut current_bands = [0.0f32; NUM_BANDS];
        let half_size = FFT_SIZE / 2;
        let log_min = 1.0f32.ln();
        let log_max = (half_size as f32).ln();

        for (i, current_band) in current_bands.iter_mut().enumerate() {
            let start_f = (log_min + (log_max - log_min) * (i as f32 / NUM_BANDS as f32)).exp();
            let end_f = (log_min + (log_max - log_min) * ((i + 1) as f32 / NUM_BANDS as f32)).exp();

            let start_bin = (start_f as usize).clamp(1, half_size - 1);
            let end_bin = (end_f as usize).clamp(start_bin + 1, half_size);

            let mut sum = 0.0;
            for sample in &buffer[start_bin..end_bin] {
                sum += sample.norm();
            }

            let count = (end_bin - start_bin).max(1) as f32;
            let magnitude = (sum / count) / (FFT_SIZE as f32 / 4.0);

            // Logarithmic compression for perceptual dynamic range
            let compressed = (1.0 + 9.0 * magnitude).log10().clamp(0.0, 1.0);

            // Temporal smoothing
            let smoothed = if compressed > self.prev_bands[i] {
                // Fast attack
                compressed
            } else {
                // Smooth decay
                self.prev_bands[i] * self.smoothing_decay
                    + compressed * (1.0 - self.smoothing_decay)
            };

            *current_band = smoothed;
            self.prev_bands[i] = smoothed;
        }

        current_bands
    }

    pub fn bands(&self) -> [f32; NUM_BANDS] {
        self.prev_bands
    }

    pub fn reset(&mut self) {
        self.prev_bands = [0.0; NUM_BANDS];
    }
}

impl Default for AudioVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_silence() {
        let mut viz = AudioVisualizer::new();
        let silence = vec![0.0f32; FFT_SIZE];
        let bands = viz.process_samples(&silence);
        for band in bands {
            assert!(
                band < 0.01,
                "Silence should yield near-zero band magnitudes"
            );
        }
    }

    #[test]
    fn test_visualizer_sine_wave() {
        let mut viz = AudioVisualizer::new();
        let freq = 440.0;
        let sample_rate = 44100.0;
        let sine: Vec<f32> = (0..FFT_SIZE)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin())
            .collect();

        let bands = viz.process_samples(&sine);
        let max_band = bands.iter().cloned().fold(0.0f32, f32::max);
        assert!(
            max_band > 0.05,
            "Sine wave must register measurable frequency energy"
        );
    }
}
