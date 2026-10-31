use super::equalizer::BiquadFilter;
use super::loudness::LoudnessNormalizer;
use super::speech_enhancer::SpeechEnhancer;
use super::{DspSettings, NUM_EQ_BANDS};

pub const EQ_FREQS: [f32; NUM_EQ_BANDS] = [
    20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0,
    200.0, 250.0, 315.0, 400.0, 500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0,
    2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0, 8000.0, 10000.0, 12500.0, 16000.0, 20000.0,
];

pub struct DspProcessor {
    sample_rate: f32,
    // 1. Bass Shelf
    bass_filter_l: BiquadFilter,
    bass_filter_r: BiquadFilter,
    // 2. Clarity High Shelf
    clarity_filter_l: BiquadFilter,
    clarity_filter_r: BiquadFilter,
    // 3. 31-Bant ISO Grafik EQ (Sol / Sağ)
    eq_filters_l: [BiquadFilter; NUM_EQ_BANDS],
    eq_filters_r: [BiquadFilter; NUM_EQ_BANDS],
    // Haas gecikme tamponu (Ambience için)
    delay_buffer: Vec<f32>,
    delay_write_pos: usize,
    delay_samples: usize,
    // 4. Akıllı Diyalog & Vokal Netleştirici
    speech_enhancer: SpeechEnhancer,
    // 5. Akıllı Algısal Loudness Normalizasyonu
    loudness_normalizer: LoudnessNormalizer,
}

impl DspProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let delay_len = (sample_rate * 0.030) as usize; // 30 ms
        let mut p = Self {
            sample_rate,
            bass_filter_l: BiquadFilter::new(),
            bass_filter_r: BiquadFilter::new(),
            clarity_filter_l: BiquadFilter::new(),
            clarity_filter_r: BiquadFilter::new(),
            eq_filters_l: [BiquadFilter::new(); NUM_EQ_BANDS],
            eq_filters_r: [BiquadFilter::new(); NUM_EQ_BANDS],
            delay_buffer: vec![0.0; delay_len],
            delay_write_pos: 0,
            delay_samples: (sample_rate * 0.012) as usize, // 12 ms Haas
            speech_enhancer: SpeechEnhancer::new(sample_rate),
            loudness_normalizer: LoudnessNormalizer::new(sample_rate),
        };
        p.update_settings(&DspSettings::default());
        p
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32, settings: &DspSettings) {
        self.sample_rate = sample_rate;
        self.delay_buffer = vec![0.0; (sample_rate * 0.030) as usize];
        self.delay_write_pos = 0;
        self.delay_samples = (sample_rate * 0.012) as usize;
        self.speech_enhancer.set_sample_rate(sample_rate);
        self.loudness_normalizer.set_sample_rate(sample_rate);
        self.update_settings(settings);
    }

    pub fn update_settings(&mut self, s: &DspSettings) {
        let sr = self.sample_rate;

        // 1. Bass Boost (80 Hz Low-shelf, 0..10 -> 0..18 dB)
        let bass_db = s.bass_boost * 1.8;
        self.bass_filter_l.set_low_shelf(sr, 80.0, 0.707, bass_db);
        self.bass_filter_r.set_low_shelf(sr, 80.0, 0.707, bass_db);

        // 2. Clarity Boost (5000 Hz High-shelf, 0..10 -> 0..15 dB)
        let clarity_db = s.clarity * 1.5;
        self.clarity_filter_l.set_high_shelf(sr, 5000.0, 0.707, clarity_db);
        self.clarity_filter_r.set_high_shelf(sr, 5000.0, 0.707, clarity_db);

        // 3. 31-Bant ISO Standart Grafik EQ (Q parametrik ayarı, varsayılan 2.8)
        let q = if s.filter_q > 0.1 { 2.8 * s.filter_q } else { 2.8 };
        for i in 0..NUM_EQ_BANDS {
            let freq = EQ_FREQS[i];
            let gain = s.eq_bands[i];
            self.eq_filters_l[i].set_peaking(sr, freq, q, gain);
            self.eq_filters_r[i].set_peaking(sr, freq, q, gain);
        }
    }

    /// Stereo ses karesini işler: (left, right) -> (out_left, out_right)
    #[inline(always)]
    pub fn process_stereo(&mut self, in_l: f32, in_r: f32, settings: &DspSettings) -> (f32, f32) {
        if !settings.enabled {
            return (in_l * settings.master_volume, in_r * settings.master_volume);
        }

        // 1. Dynamic Volume Boost (0..10 -> 0..16 dB Pre-Gain)
        let pre_gain_db = settings.dynamic_boost * 1.6;
        let pre_gain = 10.0_f32.powf(pre_gain_db / 20.0);
        let mut l = in_l * pre_gain;
        let mut r = in_r * pre_gain;

        // 2. Bass Boost
        l = self.bass_filter_l.process(l);
        r = self.bass_filter_r.process(r);

        // 3. Vektörize / Unrolled 31-Bant Grafik EQ (Döngü optimizasyonu)
        for i in (0..28).step_by(4) {
            l = self.eq_filters_l[i].process(l);
            r = self.eq_filters_r[i].process(r);
            l = self.eq_filters_l[i + 1].process(l);
            r = self.eq_filters_r[i + 1].process(r);
            l = self.eq_filters_l[i + 2].process(l);
            r = self.eq_filters_r[i + 2].process(r);
            l = self.eq_filters_l[i + 3].process(l);
            r = self.eq_filters_r[i + 3].process(r);
        }
        for i in 28..NUM_EQ_BANDS {
            l = self.eq_filters_l[i].process(l);
            r = self.eq_filters_r[i].process(r);
        }

        // 4. Clarity & Tiz Doygunluğu (Harmonic Exciter)
        l = self.clarity_filter_l.process(l);
        r = self.clarity_filter_r.process(r);
        if settings.clarity > 2.0 {
            l = soft_saturate(l, 1.0 + (settings.clarity * 0.015));
            r = soft_saturate(r, 1.0 + (settings.clarity * 0.015));
        }

        // 5. Akıllı Diyalog ve İnsan Sesi Netleştirici
        let speech_int = settings.speech_enhancement.max(
            if settings.clarity > 7.0 { (settings.clarity - 7.0) * 0.25 } else { 0.0 }
        );
        if speech_int > 0.01 {
            let (sl, sr) = self.speech_enhancer.process(l, r, speech_int);
            l = sl;
            r = sr;
        }

        // 6. Ambience (Haas Oda Yansıması)
        if settings.ambience > 0.5 {
            let delay_idx = if self.delay_write_pos >= self.delay_samples {
                self.delay_write_pos - self.delay_samples
            } else {
                self.delay_buffer.len() + self.delay_write_pos - self.delay_samples
            };
            let delayed = self.delay_buffer[delay_idx];
            let mix = (settings.ambience * 0.035).min(0.35);

            self.delay_buffer[self.delay_write_pos] = 0.5 * (l + r);
            self.delay_write_pos = (self.delay_write_pos + 1) % self.delay_buffer.len();

            l += delayed * mix;
            r += delayed * mix;
        }

        // 7. Surround Sound (Mid-Side Genişletme)
        if settings.surround > 0.5 {
            let width = 1.0 + (settings.surround * 0.12);
            let mid = 0.5 * (l + r);
            let side = 0.5 * (l - r) * width;
            l = mid + side;
            r = mid - side;
        }

        // 8. Akıllı Algısal Loudness Normalizasyonu & Fletcher-Munson
        if settings.normalization_db > 0.05 {
            let (nl, nr) = self.loudness_normalizer.process(l, r, settings.normalization_db, settings.master_volume);
            l = nl;
            r = nr;
        }

        // 9. Sol/Sağ Denge (Balance: -1.0 sol .. +1.0 sağ)
        if settings.balance.abs() > 0.01 {
            let pan_l = (1.0 - settings.balance).clamp(0.0, 1.0);
            let pan_r = (1.0 + settings.balance).clamp(0.0, 1.0);
            l *= pan_l;
            r *= pan_r;
        }

        // 10. Master Volume
        l *= settings.master_volume;
        r *= settings.master_volume;

        (l, r)
    }

    pub fn reset(&mut self) {
        self.bass_filter_l.reset();
        self.bass_filter_r.reset();
        self.clarity_filter_l.reset();
        self.clarity_filter_r.reset();
        for f in self.eq_filters_l.iter_mut() { f.reset(); }
        for f in self.eq_filters_r.iter_mut() { f.reset(); }
        self.delay_buffer.fill(0.0);
        self.speech_enhancer.reset();
        self.loudness_normalizer.reset();
    }
}

#[inline(always)]
fn soft_saturate(x: f32, drive: f32) -> f32 {
    let driven = x * drive;
    if driven > 1.0 {
        1.0 - (1.0 / (1.0 + driven - 1.0))
    } else if driven < -1.0 {
        -1.0 + (1.0 / (1.0 - driven - 1.0))
    } else {
        driven - (driven * driven * driven * 0.15)
    }
}
