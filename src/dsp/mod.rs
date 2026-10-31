pub mod autoeq;
pub mod equalizer;
pub mod effects;
pub mod limiter;
pub mod loudness;
pub mod speech_enhancer;
pub mod visualizer_data;
pub mod visualizer_analyzer;

pub use visualizer_data::VisualizerData;
pub use autoeq::HeadphoneProfile;

pub const NUM_EQ_BANDS: usize = 31;

#[derive(Debug, Clone)]
pub struct DspSettings {
    pub enabled: bool,
    pub version: u64, // Ayarlar değiştikçe artan sayaç

    // Posea 5 Ana Geliştirme Kaydırıcısı (0 - 10)
    pub clarity: f32,         // Netlik (0..10)
    pub ambience: f32,        // Ortam (0..10)
    pub surround: f32,        // Çevresel Ses (0..10)
    pub dynamic_boost: f32,   // Dinamik Güçlendirme (0..10)
    pub bass_boost: f32,      // Bass Güçlendirme (0..10)

    pub master_volume: f32,   // 0.0 - 2.0

    // 31-Bant ISO Standart Grafik EQ (-12.0 dB .. +12.0 dB)
    pub eq_bands: [f32; NUM_EQ_BANDS],

    // Gelişmiş Dinamik ve Akustik Parametreler
    pub normalization_db: f32,   // 0.0 .. 12.0 dB
    pub speech_enhancement: f32, // 0.0 .. 1.0 (Diyalog Vurgusu)
    pub balance: f32,            // -1.0 .. +1.0 (Sol - Sağ Denge)
    pub filter_q: f32,           // 0.5 .. 3.0 (Bant Genişliği)
}

impl Default for DspSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            version: 1,
            clarity: 6.0,
            ambience: 4.0,
            surround: 5.0,
            dynamic_boost: 6.0,
            bass_boost: 6.0,
            master_volume: 1.0,
            // Varsayılan Music Eğrisi (31-Bant Akıcı Eğri)
            eq_bands: [
                2.0, 3.0, 4.0, 4.5, 4.0, 3.5, 2.5, 1.5, 1.0, 0.5,
                0.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5,
                4.0, 4.5, 5.0, 5.0, 4.5, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5
            ],
            normalization_db: 0.0,
            speech_enhancement: 0.0,
            balance: 0.0,
            filter_q: 1.0,
        }
    }
}

impl DspSettings {
    pub fn apply_autoeq_profile(&mut self, p: &HeadphoneProfile) {
        self.eq_bands = p.eq_bands;
        self.clarity = p.recommended_clarity;
        self.bass_boost = p.recommended_bass;
        self.surround = p.recommended_surround;
        self.version += 1;
    }

    pub fn preset_general(&mut self) {
        self.clarity = 5.0;
        self.ambience = 2.0;
        self.surround = 3.0;
        self.dynamic_boost = 5.0;
        self.bass_boost = 4.0;
        self.speech_enhancement = 0.0;
        self.eq_bands = [
            1.0, 1.5, 2.0, 2.0, 1.5, 1.0, 0.5, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.5, 1.5, 2.0, 2.0,
            2.5, 2.5, 2.5, 2.0, 2.0, 1.5, 1.5, 1.0, 1.0, 0.5, 0.0
        ];
        self.version += 1;
    }

    pub fn preset_music(&mut self) {
        self.clarity = 6.0;
        self.ambience = 4.0;
        self.surround = 5.0;
        self.dynamic_boost = 6.0;
        self.bass_boost = 6.0;
        self.speech_enhancement = 0.0;
        self.eq_bands = [
            2.0, 3.0, 4.0, 4.5, 4.0, 3.5, 2.5, 1.5, 1.0, 0.5,
            0.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5,
            4.0, 4.5, 5.0, 5.0, 4.5, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5
        ];
        self.version += 1;
    }

    pub fn preset_voice(&mut self) {
        // Optimize Edilmiş Voice Preset Akustik Eğrisi + Speech Enhancer
        self.clarity = 7.0;
        self.ambience = 0.0;
        self.surround = 0.0;
        self.dynamic_boost = 7.0;
        self.bass_boost = 0.0;
        self.speech_enhancement = 0.85;
        self.eq_bands = [
            0.0, -1.0, -2.0, -4.0, -4.0, -3.0, -2.0, -2.0, 0.0, 1.0,
            2.0, 3.0, 4.0, 4.0, 4.0, 5.0, 5.0, 4.0, 4.0, 3.0,
            3.0, 3.0, 3.0, 3.0, 4.0, 5.0, 3.0, -1.0, -6.0, -11.0, -12.0
        ];
        self.version += 1;
    }

    pub fn preset_gaming(&mut self) {
        self.clarity = 7.0;
        self.ambience = 5.0;
        self.surround = 8.0;
        self.dynamic_boost = 7.0;
        self.bass_boost = 7.0;
        self.speech_enhancement = 0.3;
        self.eq_bands = [
            3.0, 4.0, 5.0, 5.5, 4.5, 3.0, 1.5, 0.5, 0.0, -0.5,
            -1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.5, 5.0, 5.5, 5.5,
            5.0, 4.5, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0
        ];
        self.version += 1;
    }

    pub fn preset_movies(&mut self) {
        self.clarity = 6.0;
        self.ambience = 6.0;
        self.surround = 7.0;
        self.dynamic_boost = 6.0;
        self.bass_boost = 7.0;
        self.speech_enhancement = 0.4;
        self.eq_bands = [
            3.0, 3.5, 4.5, 4.0, 3.5, 2.5, 1.5, 0.5, 0.0, 0.0,
            0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 4.5,
            4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0, -0.5, -1.0
        ];
        self.version += 1;
    }

    pub fn preset_volume_boost(&mut self) {
        self.clarity = 8.0;
        self.ambience = 2.0;
        self.surround = 4.0;
        self.dynamic_boost = 10.0;
        self.bass_boost = 7.0;
        self.speech_enhancement = 0.0;
        self.eq_bands = [
            2.5, 3.0, 3.5, 4.0, 3.5, 2.5, 2.0, 1.5, 1.0, 1.0,
            1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.0, 4.5,
            4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0, 0.0, 0.0
        ];
        self.version += 1;
    }

    pub fn preset_bass_boost(&mut self) {
        self.clarity = 4.0;
        self.ambience = 2.0;
        self.surround = 3.0;
        self.dynamic_boost = 6.0;
        self.bass_boost = 10.0;
        self.speech_enhancement = 0.0;
        self.eq_bands = [
            6.0, 7.0, 8.0, 7.5, 6.0, 4.5, 3.0, 1.5, 0.5, 0.0,
            -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.0, 2.5, 2.5, 2.0,
            2.0, 1.5, 1.5, 1.0, 1.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0
        ];
        self.version += 1;
    }

    pub fn preset_streaming(&mut self) {
        self.clarity = 6.0;
        self.ambience = 2.0;
        self.surround = 3.0;
        self.dynamic_boost = 6.0;
        self.bass_boost = 3.0;
        self.speech_enhancement = 0.2;
        self.eq_bands = [
            0.0, 0.5, 1.0, 1.5, 1.5, 1.0, 1.0, 1.5, 2.0, 2.0,
            2.5, 2.5, 3.0, 3.0, 2.5, 2.5, 2.0, 2.0, 1.5, 1.5,
            1.0, 1.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
        ];
        self.version += 1;
    }

    pub fn preset_default(&mut self) {
        self.clarity = 0.0;
        self.ambience = 0.0;
        self.surround = 0.0;
        self.dynamic_boost = 0.0;
        self.bass_boost = 0.0;
        self.speech_enhancement = 0.0;
        self.eq_bands = [0.0; NUM_EQ_BANDS];
        self.version += 1;
    }
}
