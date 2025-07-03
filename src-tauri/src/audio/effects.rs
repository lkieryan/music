 /// 音效处理器
pub struct AudioEffects {
    pub equalizer: Equalizer,
    pub reverb: Reverb,
}

/// 均衡器
pub struct Equalizer {
    pub bands: Vec<f32>, // 各频段增益
}

impl Equalizer {
    pub fn new() -> Self {
        Self {
            bands: vec![0.0; 10], // 10段均衡器
        }
    }

    pub fn set_band(&mut self, index: usize, gain: f32) {
        if index < self.bands.len() {
            self.bands[index] = gain;
        }
    }

    pub fn reset(&mut self) {
        self.bands.fill(0.0);
    }
}

/// 混响效果
pub struct Reverb {
    pub enabled: bool,
    pub level: f32,
}

impl Reverb {
    pub fn new() -> Self {
        Self {
            enabled: false,
            level: 0.0,
        }
    }

    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 1.0);
    }
}