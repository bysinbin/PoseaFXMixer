use std::sync::Arc;
use realfft::{RealFftPlanner, RealToComplex, num_complex::Complex};
use crate::dsp::visualizer_data::{VisualizerData, NUM_SPECTRUM_BARS};

const FFT_SIZE: usize = 512;

pub struct VisualizerAnalyzer {
    r2c: Arc<dyn RealToComplex<f32>>,
    input_buf: Vec<f32>,
    fft_in: Vec<f32>,
    fft_out: Vec<Complex<f32>>,
    window: Vec<f32>,
    buf_pos: usize,
    visualizer: Arc<VisualizerData>,
    band_ranges: [(usize, usize); NUM_SPECTRUM_BARS],
}

impl VisualizerAnalyzer {
    pub fn new(visualizer: Arc<VisualizerData>) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let r2c = planner.plan_fft_forward(FFT_SIZE);

        let mut window = Vec::with_capacity(FFT_SIZE);
        for i in 0..FFT_SIZE {
            // Hann window
            let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos());
            window.push(w);
        }

        // 64 logaritmik frekans band aralığı (bin 1'den 230'a kadar)
        let mut band_ranges = [(1usize, 1usize); NUM_SPECTRUM_BARS];
        let min_bin = 1.0f32;
        let max_bin = 230.0f32;
        let factor = (max_bin / min_bin).powf(1.0 / (NUM_SPECTRUM_BARS as f32));

        let mut cur_low = min_bin;
        for i in 0..NUM_SPECTRUM_BARS {
            let next = (cur_low * factor).max(cur_low + 0.8);
            let b_start = (cur_low as usize).clamp(1, 240);
            let b_end = ((next as usize).max(b_start)).clamp(1, 240);
            band_ranges[i] = (b_start, b_end);
            cur_low = next;
        }

        Self {
            fft_in: r2c.make_input_vec(),
            fft_out: r2c.make_output_vec(),
            r2c,
            input_buf: vec![0.0; FFT_SIZE],
            window,
            buf_pos: 0,
            visualizer,
            band_ranges,
        }
    }

    #[inline(always)]
    pub fn add_sample(&mut self, l: f32, r: f32) {
        self.visualizer.update_peak(l, r);

        self.input_buf[self.buf_pos] = 0.5 * (l + r);
        self.buf_pos += 1;

        if self.buf_pos >= FFT_SIZE {
            self.process_fft();
            self.buf_pos = 0;
        }
    }

    fn process_fft(&mut self) {
        for i in 0..FFT_SIZE {
            self.fft_in[i] = self.input_buf[i] * self.window[i];
        }

        if self.r2c.process(&mut self.fft_in, &mut self.fft_out).is_ok() {
            let mut bands = [0.0f32; NUM_SPECTRUM_BARS];

            for (band_idx, &(start_bin, end_bin)) in self.band_ranges.iter().enumerate() {
                let mut sum_mag = 0.0f32;
                let count = (end_bin - start_bin + 1) as f32;

                for k in start_bin..=end_bin {
                    if k < self.fft_out.len() {
                        let c = self.fft_out[k];
                        let mag = (c.re * c.re + c.im * c.im).sqrt();
                        sum_mag += mag;
                    }
                }

                let avg_mag = sum_mag / count;
                let norm = avg_mag / (FFT_SIZE as f32 * 0.18);
                let db = if norm > 1e-4 {
                    20.0 * norm.log10()
                } else {
                    -50.0
                };
                let level = ((db + 42.0) / 42.0).clamp(0.0, 1.0);
                bands[band_idx] = level;
            }

            self.visualizer.update_bands(&bands);
        }
    }
}
