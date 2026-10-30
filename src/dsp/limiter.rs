pub struct FastLimiter {
    sample_rate: f32,
    envelope: f32,
    attack_coeff: f32,
    release_coeff: f32,
    threshold: f32, // -0.2 dBFS (~0.977)
}

impl FastLimiter {
    pub fn new(sample_rate: f32) -> Self {
        let mut limiter = Self {
            sample_rate,
            envelope: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            threshold: 0.977,
        };
        limiter.set_coefficients(0.5, 50.0); // 0.5 ms attack, 50 ms release
        limiter
    }

    pub fn set_coefficients(&mut self, attack_ms: f32, release_ms: f32) {
        let attack_time = attack_ms * 0.001;
        let release_time = release_ms * 0.001;
        self.attack_coeff = (-1.0 / (attack_time * self.sample_rate)).exp();
        self.release_coeff = (-1.0 / (release_time * self.sample_rate)).exp();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.set_coefficients(0.5, 50.0);
    }

    #[inline(always)]
    pub fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        let peak = l.abs().max(r.abs());

        // Envelope takibi
        if peak > self.envelope {
            self.envelope = self.attack_coeff * self.envelope + (1.0 - self.attack_coeff) * peak;
        } else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * peak;
        }

        let gain = if self.envelope > self.threshold {
            self.threshold / self.envelope
        } else {
            1.0
        };

        // Sert kırpılmayı (hard clip) engellemek için son güvenlik emniyeti
        let out_l = (l * gain).clamp(-0.999, 0.999);
        let out_r = (r * gain).clamp(-0.999, 0.999);

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }
}
