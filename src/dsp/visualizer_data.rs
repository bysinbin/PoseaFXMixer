use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub const NUM_SPECTRUM_BARS: usize = 64;

pub struct VisualizerData {
    // 64 frekans bandı için tepe seviyeleri (0.0 - 1.0)
    bands: [AtomicU32; NUM_SPECTRUM_BARS],
    pub master_peak: AtomicU32,
}

impl VisualizerData {
    pub fn new() -> Arc<Self> {
        const INIT_ZERO: AtomicU32 = AtomicU32::new(0);
        Arc::new(Self {
            bands: [INIT_ZERO; NUM_SPECTRUM_BARS],
            master_peak: AtomicU32::new(0),
        })
    }

    #[inline(always)]
    pub fn update_peak(&self, l: f32, r: f32) {
        let peak = l.abs().max(r.abs()).min(1.0);
        let current_bits = self.master_peak.load(Ordering::Relaxed);
        let current = f32::from_bits(current_bits);
        let new_val = if peak > current {
            peak
        } else {
            current * 0.90
        };
        self.master_peak.store(new_val.to_bits(), Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn update_bands(&self, band_levels: &[f32; NUM_SPECTRUM_BARS]) {
        for (i, &lvl) in band_levels.iter().enumerate() {
            let clamped = lvl.clamp(0.0, 1.0);
            let cur_bits = self.bands[i].load(Ordering::Relaxed);
            let cur = f32::from_bits(cur_bits);
            let next = if clamped > cur {
                clamped
            } else {
                cur * 0.85 // Akıcı ve doğal düşüş
            };
            self.bands[i].store(next.to_bits(), Ordering::Relaxed);
        }
    }

    pub fn get_bands(&self) -> [f32; NUM_SPECTRUM_BARS] {
        let mut res = [0.0; NUM_SPECTRUM_BARS];
        for i in 0..NUM_SPECTRUM_BARS {
            res[i] = f32::from_bits(self.bands[i].load(Ordering::Relaxed));
        }
        res
    }

    pub fn get_master_peak(&self) -> f32 {
        f32::from_bits(self.master_peak.load(Ordering::Relaxed))
    }
}
