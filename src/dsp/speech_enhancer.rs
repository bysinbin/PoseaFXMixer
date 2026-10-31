use std::f32::consts::PI;

/// Akıllı Diyalog ve İnsan Sesi Netleştirici (Speech & Dialogue Intelligence Enhancer)
/// Formant frekanslarını (1.2 kHz - 3.8 kHz) ve harf anlaşılırlığını dinamik olarak yükseltir.
#[derive(Debug, Clone)]
pub struct SpeechEnhancer {
    sample_rate: f32,
    // 2. Derece Peaking Filtre Katsayıları (1.8 kHz merkezli)
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // Durum değişkenleri (Stereo)
    x1_l: f32, x2_l: f32, y1_l: f32, y2_l: f32,
    x1_r: f32, x2_r: f32, y1_r: f32, y2_r: f32,
}

impl SpeechEnhancer {
    pub fn new(sample_rate: f32) -> Self {
        let mut s = Self {
            sample_rate,
            b0: 1.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
            x1_l: 0.0, x2_l: 0.0, y1_l: 0.0, y2_l: 0.0,
            x1_r: 0.0, x2_r: 0.0, y1_r: 0.0, y2_r: 0.0,
        };
        s.update_coefficients(3.5); // Varsayılan +3.5 dB diyalog vurgusu
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coefficients(3.5);
    }

    fn update_coefficients(&mut self, gain_db: f32) {
        let f0 = 2200.0; // İnsan sesinin en kritik anlaşılırlık formants merkezi
        let q = 1.2;
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * f0 / self.sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha / a;
        self.b0 = (1.0 + alpha * a) / a0;
        self.b1 = (-2.0 * cos_w0) / a0;
        self.b2 = (1.0 - alpha * a) / a0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha / a) / a0;
    }

    /// Stereo sample işleme
    /// `intensity`: 0.0 (kapalı) .. 1.0 (tam güç)
    #[inline(always)]
    pub fn process(&mut self, left: f32, right: f32, intensity: f32) -> (f32, f32) {
        if intensity <= 0.01 {
            return (left, right);
        }

        // Sol Kanal Biquad
        let y_l = self.b0 * left + self.b1 * self.x1_l + self.b2 * self.x2_l - self.a1 * self.y1_l - self.a2 * self.y2_l;
        self.x2_l = self.x1_l; self.x1_l = left;
        self.y2_l = self.y1_l; self.y1_l = y_l;

        // Sağ Kanal Biquad
        let y_r = self.b0 * right + self.b1 * self.x1_r + self.b2 * self.x2_r - self.a1 * self.y1_r - self.a2 * self.y2_r;
        self.x2_r = self.x1_r; self.x1_r = right;
        self.y2_r = self.y1_r; self.y1_r = y_r;

        // Hafif harmonik varlık zenginleştirmesi (Vokal sıcaklığı)
        let exc_l = y_l + (y_l.abs() * y_l) * 0.06;
        let exc_r = y_r + (y_r.abs() * y_r) * 0.06;

        let out_l = left * (1.0 - intensity) + exc_l * intensity;
        let out_r = right * (1.0 - intensity) + exc_r * intensity;

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.x1_l = 0.0; self.x2_l = 0.0; self.y1_l = 0.0; self.y2_l = 0.0;
        self.x1_r = 0.0; self.x2_r = 0.0; self.y1_r = 0.0; self.y2_r = 0.0;
    }
}
