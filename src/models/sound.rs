use crate::wasm_bindgen;
use std::collections::HashMap;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HitSoundType {
    Normal,
    Clap,
    Whistle,
    Finish,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SoundEffect {
    #[wasm_bindgen(getter_with_clone)]
    pub time: i32,
    #[wasm_bindgen(getter_with_clone)]
    pub volume: u8,
    #[wasm_bindgen(getter_with_clone)]
    pub sample: usize,
}

#[wasm_bindgen]
impl SoundEffect {
    #[inline]
    #[wasm_bindgen(constructor)]
    pub fn new(time: i32, volume: u8, sample: usize) -> Self {
        Self { time, volume, sample }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct KeySound {
    #[wasm_bindgen(getter_with_clone)]
    pub volume: u8,
    #[wasm_bindgen(getter_with_clone)]
    pub hitsound_type: HitSoundType,
    #[wasm_bindgen(getter_with_clone)]
    pub sample: Option<usize>,
    #[wasm_bindgen(getter_with_clone)]
    pub has_custom: bool,
}

impl KeySound {
    pub fn of_type(volume: u8, hitsound_type: HitSoundType) -> Self  {
        Self {
            volume,
            hitsound_type,
            sample: None,
            has_custom: false
        }
    }

    pub fn normal(volume: u8) -> Self {
        Self {
            volume,
            hitsound_type: HitSoundType::Normal,
            sample: None,
            has_custom: false
        }
    }

    pub fn clap(volume: u8) -> Self {
        Self {
            volume,
            hitsound_type: HitSoundType::Clap,
            sample: None,
            has_custom: false
        }
    }

    pub fn whistle(volume: u8) -> Self {
        Self {
            volume,
            hitsound_type: HitSoundType::Whistle,
            sample: None,
            has_custom: false
        }
    }

    pub fn finish(volume: u8) -> Self {
        Self {
            volume,
            hitsound_type: HitSoundType::Finish,
            sample: None,
            has_custom: false
        }
    }

    pub fn with_custom(volume: u8, sample_index: usize, hitsound_type: Option<HitSoundType>) -> Self {
        let hstype = if let Some(hstype) = hitsound_type {
            hstype
        } else {
            HitSoundType::Normal
        };
        Self {
            volume,
            hitsound_type: hstype,
            sample: Some(sample_index),
            has_custom: true
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct KeySoundRow {
    sounds: Vec<KeySound>,
    pub is_empty: bool,
}

impl KeySoundRow {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sounds: vec![KeySound::normal(100); capacity],
            is_empty: true,
        }
    }

    pub fn empty() -> Self {
        Self {
            sounds: Vec::new(),
            is_empty: true,
        }
    }
    
    pub fn with(sounds: Vec<KeySound>) -> Self {
        Self {
            sounds,
            is_empty: false,
        }
    }

    pub fn with_unwrap(sounds: &[Option<KeySound>]) -> Self {
        Self {
            sounds: sounds
                .iter()
                .map(|s| s.unwrap_or(KeySound::normal(100)))
                .collect(),
            is_empty: false,
        }
    }
    
    pub fn get_sounds(&self) -> &Vec<KeySound> {
        &self.sounds
    }

    pub fn len(&self) -> usize {
        self.sounds.len()
    }
    
    pub fn as_ptr(&mut self) -> *const KeySound {
        self.sounds.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut KeySound {
        self.sounds.as_mut_ptr()
    }
}

impl std::ops::Index<usize> for KeySoundRow {
    type Output = KeySound;

    fn index(&self, index: usize) -> &Self::Output {
        &self.sounds[index]
    }
}

impl std::ops::IndexMut<usize> for KeySoundRow {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.sounds[index]
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SoundBank {
    #[wasm_bindgen(getter_with_clone)]
    pub audio_tracks: Vec<String>,
    sound_sample_paths: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub sound_effects: Vec<SoundEffect>,
    sample_map: HashMap<String, usize>,
}

#[wasm_bindgen]
impl SoundBank {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            audio_tracks: Vec::new(),
            sound_sample_paths: Vec::new(),
            sound_effects: Vec::new(),
            sample_map: HashMap::new(),
        }
    }

    #[wasm_bindgen]
    pub fn add_sound_sample(&mut self, path: String) -> usize {
        if let Some(&index) = self.sample_map.get(&path) {
            index
        } else {
            let index = self.sound_sample_paths.len();
            self.sound_sample_paths.push(path.clone());
            self.sample_map.insert(path, index);
            index
        }
    }

    #[wasm_bindgen]
    pub fn add_sound_sample_with_index(&mut self, index: usize, path: String) {
        if path.is_empty() {
            return;
        }

        if index >= self.sound_sample_paths.len() {
            self.sound_sample_paths.resize(index + 1, String::new());
        }
        
        if !self.sound_sample_paths[index].is_empty() {
            self.sample_map.remove(&self.sound_sample_paths[index]);
        }
        
        self.sound_sample_paths[index] = path.clone();
        self.sample_map.insert(path, index);
    }

    pub fn add_sound_effect(&mut self, sound_effect: SoundEffect) {
        self.sound_effects.push(sound_effect);
    }

    #[wasm_bindgen]
    pub fn get_sound_sample(&self, index: usize) -> Option<String> {
        self.sound_sample_paths.get(index)
            .filter(|s| !s.is_empty())  
            .cloned()
    }

    #[wasm_bindgen]
    pub fn get_index_sample(&self, sample_path: &str) -> Option<usize> {
        self.sample_map.get(sample_path).copied()
    }

    #[wasm_bindgen]
    pub fn get_sample_paths(&self) -> Vec<String> {
        self.sound_sample_paths.clone()
    }

    #[wasm_bindgen]
    pub fn contains_path(&self, path: &str) -> bool {
        self.sample_map.contains_key(path)
    }

    #[wasm_bindgen]
    pub fn sample_count(&self) -> usize {
        self.sound_sample_paths.iter().filter(|s| !s.is_empty()).count()
    }

    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.sample_map.is_empty()
    }
}