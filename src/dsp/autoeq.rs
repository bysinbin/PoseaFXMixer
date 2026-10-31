use crate::dsp::NUM_EQ_BANDS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadphoneCategory {
    Audiophile,
    ConsumerWireless,
    Gaming,
    InEar,
}

#[derive(Debug, Clone)]
pub struct HeadphoneProfile {
    pub id: &'static str,
    pub brand: &'static str,
    pub model: &'static str,
    pub category: HeadphoneCategory,
    pub description: &'static str,
    pub eq_bands: [f32; NUM_EQ_BANDS],
    pub recommended_clarity: f32,
    pub recommended_bass: f32,
    pub recommended_surround: f32,
}

pub struct AutoEqDatabase;

impl AutoEqDatabase {
    pub fn get_profiles() -> &'static [HeadphoneProfile] {
        &PROFILES
    }

    pub fn find_by_id(id: &str) -> Option<&'static HeadphoneProfile> {
        PROFILES.iter().find(|p| p.id == id)
    }
}

// 31 ISO Standart Frekansları (Hz):
// [20, 25, 31.5, 40, 50, 63, 80, 100, 125, 160, 200, 250, 315, 400, 500, 630, 800, 1000, 1250, 1600, 2000, 2500, 3150, 4000, 5000, 6300, 8000, 10000, 12500, 16000, 20000]

static PROFILES: [HeadphoneProfile; 14] = [
    // 1. Sony WH-1000XM5
    HeadphoneProfile {
        id: "sony_wh1000xm5",
        brand: "Sony",
        model: "WH-1000XM5",
        category: HeadphoneCategory::ConsumerWireless,
        description: "Harman Target düzeltmesi: Aşırı alt bası dengeler, üst mid ve tiz ferahlığını açar.",
        eq_bands: [
            -1.0, -1.5, -2.0, -2.5, -2.0, -1.0, -0.5, 0.0, 0.0, 0.5,
            0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 2.5, 2.0,
            1.5, 2.0, 3.5, 4.0, 3.5, 2.0, 1.5, 2.0, 2.5, 2.0, 1.0,
        ],
        recommended_clarity: 6.5,
        recommended_bass: 4.0,
        recommended_surround: 5.0,
    },
    // 2. Sony WH-1000XM4
    HeadphoneProfile {
        id: "sony_wh1000xm4",
        brand: "Sony",
        model: "WH-1000XM4",
        category: HeadphoneCategory::ConsumerWireless,
        description: "150-250Hz çamurlu bas tepesini temizler, vokalleri öne çıkarır.",
        eq_bands: [
            -0.5, -1.0, -1.5, -2.0, -2.5, -3.0, -3.0, -2.5, -1.5, -1.0,
            0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 3.0, 2.5,
            2.0, 2.5, 3.0, 3.5, 4.0, 3.0, 1.5, 2.0, 2.5, 1.5, 0.5,
        ],
        recommended_clarity: 7.0,
        recommended_bass: 4.5,
        recommended_surround: 4.5,
    },
    // 3. Apple AirPods Pro 2
    HeadphoneProfile {
        id: "apple_airpods_pro_2",
        brand: "Apple",
        model: "AirPods Pro 2",
        category: HeadphoneCategory::InEar,
        description: "Doğal Harman referans ayarı: Geniş sahne, tatlı sub-bas ve şeffaf tizler.",
        eq_bands: [
            1.5, 1.5, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.5, 1.5, 1.0,
            0.5, 0.0, -0.5, -1.0, 0.0, 1.0, 2.0, 1.5, 1.0, 0.5, 0.0,
        ],
        recommended_clarity: 5.5,
        recommended_bass: 5.0,
        recommended_surround: 6.0,
    },
    // 4. Apple AirPods Max
    HeadphoneProfile {
        id: "apple_airpods_max",
        brand: "Apple",
        model: "AirPods Max",
        category: HeadphoneCategory::ConsumerWireless,
        description: "2-4kHz bandındaki çekilmeyi onarır, orkestral derinliği parlatır.",
        eq_bands: [
            0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 3.0, 2.5,
            2.0, 2.0, 2.5, 2.5, 2.0, 1.5, 1.0, 1.5, 2.0, 1.5, 0.5,
        ],
        recommended_clarity: 6.0,
        recommended_bass: 5.0,
        recommended_surround: 6.5,
    },
    // 5. Sennheiser HD 600 / HD 650
    HeadphoneProfile {
        id: "sennheiser_hd600",
        brand: "Sennheiser",
        model: "HD 600 / HD 650",
        category: HeadphoneCategory::Audiophile,
        description: "Efsanevi açık odyofil referansı: 20-60Hz sub-bas dolgunluğu ve açık hava tizi ekler.",
        eq_bands: [
            6.5, 6.0, 5.5, 4.5, 3.5, 2.5, 1.5, 0.5, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5,
            0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 3.5, 2.5, 1.5,
        ],
        recommended_clarity: 5.0,
        recommended_bass: 7.0,
        recommended_surround: 4.0,
    },
    // 6. Sennheiser HD 560S
    HeadphoneProfile {
        id: "sennheiser_hd560s",
        brand: "Sennheiser",
        model: "HD 560S",
        category: HeadphoneCategory::Audiophile,
        description: "Geniş sahne, 5.5kHz sertliğini yumuşatır ve alt bas uzantısını güçlendirir.",
        eq_bands: [
            4.5, 4.0, 3.5, 3.0, 2.0, 1.0, 0.5, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, -0.5,
            -1.0, -1.5, -2.0, -1.5, 0.0, 1.5, 2.0, 1.5, 1.0, 0.5, 0.0,
        ],
        recommended_clarity: 5.5,
        recommended_bass: 6.0,
        recommended_surround: 5.0,
    },
    // 7. Beyerdynamic DT 770 Pro (80 Ohm)
    HeadphoneProfile {
        id: "beyer_dt770",
        brand: "Beyerdynamic",
        model: "DT 770 Pro (80Ω)",
        category: HeadphoneCategory::Audiophile,
        description: "Meşhur 6-8kHz 'Beyer tiz sivrilmesini' dizginler, orta sesleri ısıtır.",
        eq_bands: [
            -1.0, -1.0, -0.5, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, -0.5,
            -1.0, -1.0, -0.5, 0.5, 1.5, 2.0, 2.5, 3.0, 2.5, 1.5,
            0.5, -0.5, -2.0, -3.5, -4.5, -4.0, -2.0, 0.5, 1.5, 1.0, 0.0,
        ],
        recommended_clarity: 4.5,
        recommended_bass: 5.0,
        recommended_surround: 4.0,
    },
    // 8. Beyerdynamic DT 990 Pro (250 Ohm)
    HeadphoneProfile {
        id: "beyer_dt990",
        brand: "Beyerdynamic",
        model: "DT 990 Pro (250Ω)",
        category: HeadphoneCategory::Audiophile,
        description: "Açık stüdyo miks: Aşırı tiz tepe noktasını ehlileştirip gövdeli bas derinliği verir.",
        eq_bands: [
            4.0, 3.5, 3.0, 2.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.5,
            1.0, 1.5, 2.0, 2.5, 3.0, 3.0, 2.5, 2.0, 1.0, 0.0,
            -1.0, -2.5, -4.5, -5.5, -6.0, -4.5, -2.0, 0.0, 1.0, 0.5, 0.0,
        ],
        recommended_clarity: 4.0,
        recommended_bass: 6.5,
        recommended_surround: 4.5,
    },
    // 9. Audio-Technica ATH-M50x
    HeadphoneProfile {
        id: "audiotechnica_m50x",
        brand: "Audio-Technica",
        model: "ATH-M50x",
        category: HeadphoneCategory::Audiophile,
        description: "Kapalı DJ/Stüdyo: 200Hz gölgesini siler, sıkışık sahneyi genişletir.",
        eq_bands: [
            -0.5, -0.5, 0.0, 0.0, -0.5, -1.0, -1.5, -2.0, -2.0, -1.5,
            -0.5, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 3.0, 2.0,
            1.0, 0.0, -1.0, -1.5, -1.0, 0.5, 1.5, 2.0, 1.5, 0.5, 0.0,
        ],
        recommended_clarity: 6.0,
        recommended_bass: 4.5,
        recommended_surround: 6.5,
    },
    // 10. SteelSeries Arctis Nova Pro / Arctis 7
    HeadphoneProfile {
        id: "steelseries_arctis_nova",
        brand: "SteelSeries",
        model: "Arctis Nova Pro / 7",
        category: HeadphoneCategory::Gaming,
        description: "Rekabetçi Oyun: Ayak seslerini (2-4kHz) öne çıkarır, patlamaları sıkılaştırır.",
        eq_bands: [
            1.0, 1.5, 2.0, 2.0, 1.5, 1.0, 0.5, 0.0, -0.5, -1.0,
            -0.5, 0.5, 1.0, 1.5, 2.0, 2.5, 3.5, 4.5, 5.0, 4.5,
            4.0, 3.5, 3.0, 2.5, 2.0, 2.0, 2.5, 3.0, 2.0, 1.0, 0.0,
        ],
        recommended_clarity: 7.5,
        recommended_bass: 5.5,
        recommended_surround: 7.0,
    },
    // 11. Razer BlackShark V2 Pro
    HeadphoneProfile {
        id: "razer_blackshark_v2",
        brand: "Razer",
        model: "BlackShark V2 Pro",
        category: HeadphoneCategory::Gaming,
        description: "Espor FPS: Boğuk alt bası temizler, silah sesleri ve yön tayinini keskinleştirir.",
        eq_bands: [
            -1.0, -1.0, -0.5, 0.0, -0.5, -1.5, -2.0, -2.0, -1.5, -0.5,
            0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 4.5, 4.5, 4.0,
            3.5, 3.0, 2.5, 2.0, 1.5, 2.0, 2.5, 2.0, 1.5, 0.5, 0.0,
        ],
        recommended_clarity: 8.0,
        recommended_bass: 4.5,
        recommended_surround: 7.5,
    },
    // 12. HyperX Cloud II / Cloud Alpha
    HeadphoneProfile {
        id: "hyperx_cloud_2",
        brand: "HyperX",
        model: "Cloud II / Alpha",
        category: HeadphoneCategory::Gaming,
        description: "Klasik Oyuncu: Metalik tizleri ipeksi yapar, bas darbesini sıkılaştırır.",
        eq_bands: [
            2.0, 2.0, 1.5, 1.0, 0.5, 0.0, -0.5, -1.0, -1.5, -1.0,
            0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 3.0, 2.0,
            1.0, 0.5, -0.5, -1.5, -2.5, -1.5, 0.0, 1.5, 1.0, 0.5, 0.0,
        ],
        recommended_clarity: 6.5,
        recommended_bass: 6.0,
        recommended_surround: 6.0,
    },
    // 13. Bose QuietComfort 35 II / QC45
    HeadphoneProfile {
        id: "bose_qc35_qc45",
        brand: "Bose",
        model: "QuietComfort 35 / 45",
        category: HeadphoneCategory::ConsumerWireless,
        description: "Seyahat & Konfor: ANC'nin sebep olduğu yapay orta frekans boşluğunu doldurur.",
        eq_bands: [
            1.0, 1.0, 1.5, 1.5, 1.0, 0.5, 0.0, -0.5, -0.5, 0.0,
            0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.0, 2.5, 2.0, 1.5,
            1.0, 1.5, 2.0, 2.5, 2.0, 1.5, 1.0, 1.5, 1.5, 1.0, 0.0,
        ],
        recommended_clarity: 6.0,
        recommended_bass: 5.5,
        recommended_surround: 5.5,
    },
    // 14. Samsung Galaxy Buds 2 Pro
    HeadphoneProfile {
        id: "samsung_buds2_pro",
        brand: "Samsung",
        model: "Galaxy Buds 2 Pro",
        category: HeadphoneCategory::InEar,
        description: "Kristal Harman Eğrisi: Mükemmel tonal denge, derin bas ve parlak sahne.",
        eq_bands: [
            1.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.5, 2.0, 2.0, 1.5,
            1.0, 0.5, 0.0, -0.5, 0.0, 1.0, 1.5, 1.0, 0.5, 0.0, 0.0,
        ],
        recommended_clarity: 5.5,
        recommended_bass: 5.0,
        recommended_surround: 5.0,
    },
];
