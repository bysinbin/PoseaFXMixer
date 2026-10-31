/// Profesyonel Lookahead Brickwall Limiter & Çift Aşamalı (Dual-Release) Tepe Koruyucu
/// 2.5 ms önden okuma (lookahead) gecikmesi ile ani tepe noktalarını kırpılma olmadan yumuşakça sınırlar.

const LOOKAHEAD_SIZE: usize = 256;

pub struct FastLimiter {
    sample_rate: f32,
    threshold: f32, // 0.9772 (-0.2 dBFS)
    lookahead_samples: usize,
    buffer_l: [f32; LOOKAHEAD_SIZE],
    buffer_r: [f32; LOOKAHEAD_SIZE],
    write_pos: usize,

    envelope: f32,
    release_coeff_fast: f32,
    release_coeff_slow: f32,
    current_gain: f32,
}

impl FastLimiter {
    pub fn new(sample_rate: f32) -> Self {
        let mut limiter = Self {
            sample_rate,
            threshold: 0.9772, // -0.2 dBFS tavan
            lookahead_samples: 120, // ~2.5 ms (48 kHz)
            buffer_l: [0.0; LOOKAHEAD_SIZE],
            buffer_r: [0.0; LOOKAHEAD_SIZE],
            write_pos: 0,
            envelope: 0.0,
            release_coeff_fast: 0.0,
            release_coeff_slow: 0.0,
            current_gain: 1.0,
        };
        limiter.update_coeffs();
        limiter
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coeffs();
    }

    fn update_coeffs(&mut self) {
        // 2.5 ms önden okuma (lookahead)
        self.lookahead_samples = ((self.sample_rate * 0.0025) as usize).clamp(32, LOOKAHEAD_SIZE - 1);
        // Hızlı sönümlenme (davul ve ani vuruşlar için): 40 ms
        let fast_t = 0.040;
        self.release_coeff_fast = (-1.0 / (fast_t * self.sample_rate)).exp();
        // Yavaş sönümlenme (bas titreşimleri ve genel müzik için): 180 ms
        let slow_t = 0.180;
        self.release_coeff_slow = (-1.0 / (slow_t * self.sample_rate)).exp();
    }

    #[inline(always)]
    pub fn process_stereo(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let peak = in_l.abs().max(in_r.abs());

        // 1. İleriye Dönük Zirve Tespiti (Peak Detection)
        if peak > self.envelope {
            self.envelope = peak;
        } else {
            // Bas dalga formu modülasyonunu engellemek için çift aşamalı program-bağımlı sönümleme
            let coeff = if self.envelope > 1.25 {
                self.release_coeff_fast
            } else {
                self.release_coeff_slow
            };
            self.envelope = self.envelope * coeff + peak * (1.0 - coeff);
        }

        // 2. Hedef Kazanç Hesaplama (Target Gain Reduction)
        let target_gain = if self.envelope > self.threshold {
            self.threshold / self.envelope
        } else {
            1.0
        };

        // 3. Kazanç Yumuşatma (Gain Smoothing)
        if target_gain < self.current_gain {
            self.current_gain += (target_gain - self.current_gain) * 0.22;
        } else {
            self.current_gain += (target_gain - self.current_gain) * 0.0025;
        }

        // 4. Lookahead Gecikme Tamponundan Örnekleri Oku
        let read_pos = (self.write_pos + LOOKAHEAD_SIZE - self.lookahead_samples) & (LOOKAHEAD_SIZE - 1);
        let del_l = self.buffer_l[read_pos];
        let del_r = self.buffer_r[read_pos];

        // Yeni örnekleri tampona yaz
        self.buffer_l[self.write_pos] = in_l;
        self.buffer_r[self.write_pos] = in_r;
        self.write_pos = (self.write_pos + 1) & (LOOKAHEAD_SIZE - 1);

        // 5. Kazancı Uygula
        let limited_l = del_l * self.current_gain;
        let limited_r = del_r * self.current_gain;

        // 6. Emniyetli Yumuşak Tavan (Soft-Knee Ceiling):
        // Asla sert kırpma (hard clip) yapmaz; pürüzsüz analog bant doygunluğu ile ses asla çatlamaz
        let out_l = soft_knee_ceiling(limited_l, self.threshold);
        let out_r = soft_knee_ceiling(limited_r, self.threshold);

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_pos = 0;
        self.envelope = 0.0;
        self.current_gain = 1.0;
    }
}

#[inline(always)]
fn soft_knee_ceiling(x: f32, ceiling: f32) -> f32 {
    let knee = 0.88;
    let abs_x = x.abs();
    if abs_x <= knee {
        x
    } else {
        let sign = x.signum();
        let excess = (abs_x - knee) / (1.5 - knee);
        let sat = knee + (ceiling - knee) * excess.tanh();
        sign * sat.min(ceiling)
    }
}
