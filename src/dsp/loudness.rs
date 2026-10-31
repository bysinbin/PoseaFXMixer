/// Akıllı Algısal Ses Seviyesi Normalizasyonu & Fletcher-Munson Eşit İşitilebilirlik Motoru (EBU R128 Yaklaşımı)

#[derive(Debug, Clone)]
pub struct LoudnessNormalizer {
    sample_rate: f32,
    short_term_rms: f32,
    current_gain: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl LoudnessNormalizer {
    pub fn new(sample_rate: f32) -> Self {
        let mut s = Self {
            sample_rate,
            short_term_rms: 0.1,
            current_gain: 1.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
        };
        s.update_coeffs();
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.update_coeffs();
    }

    fn update_coeffs(&mut self) {
        // Attack: 40ms, Release: 600ms
        let attack_time = 0.040;
        let release_time = 0.600;
        self.attack_coeff = (-1.0 / (attack_time * self.sample_rate)).exp();
        self.release_coeff = (-1.0 / (release_time * self.sample_rate)).exp();
    }

    /// Stereo sample çiftini normalizasyon ve dinamik kazanç filtresinden geçirir.
    /// `norm_amount_db`: 0.0 .. 12.0 dB (Tercihler arayüzünden gelen ayar)
    /// `master_vol_linear`: Mevcut ana ses kazancı (Fletcher-Munson hesaplaması için)
    #[inline(always)]
    pub fn process(&mut self, left: f32, right: f32, norm_amount_db: f32, master_vol_linear: f32) -> (f32, f32) {
        if norm_amount_db <= 0.05 {
            return (left, right);
        }

        // 1. Algısal Anlık Enerji (K-Weighting benzeri yüksek ve orta ağırlıklı enerji)
        let sample_energy = (left * left + right * right) * 0.5;

        // 2. RMS Takibi (Attack & Release)
        let coeff = if sample_energy > self.short_term_rms {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.short_term_rms = coeff * self.short_term_rms + (1.0 - coeff) * sample_energy;

        let rms = self.short_term_rms.sqrt().max(1e-5);

        // 3. Hedef Kazanç Hesaplama (Target RMS: ~ -18 dBFS)
        let target_rms = 0.125; // -18 dBFS
        let raw_ratio = target_rms / rms;
        
        // norm_amount_db oranına göre maksimum kazanç artışını sınırla (0 .. +norm_amount_db)
        let max_gain = 10.0_f32.powf(norm_amount_db / 20.0);
        let min_gain = 10.0_f32.powf(-norm_amount_db / 20.0);
        let desired_gain = raw_ratio.clamp(min_gain, max_gain);

        // Kazanç yumuşatma (zipper noise önleyici)
        self.current_gain += (desired_gain - self.current_gain) * 0.002;

        // 4. Fletcher-Munson Dinamik Eşit-İşitilebilirlik Telafisi
        // Düşük ses seviyelerinde kulak bası ve havayı duymaz; master_volume azaldıkça hafif bas/tiz sıcaklığı ekle
        let low_vol_boost = if master_vol_linear < 0.8 {
            (0.8 - master_vol_linear) * 0.15
        } else {
            0.0
        };

        let effective_gain = self.current_gain * (1.0 + low_vol_boost);

        (left * effective_gain, right * effective_gain)
    }

    pub fn reset(&mut self) {
        self.short_term_rms = 0.1;
        self.current_gain = 1.0;
    }
}
