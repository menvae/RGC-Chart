use std::fmt;
use crate::wasm_bindgen;

def_varied_type_enum!(pub ChartDefaults {
    TITLE: &'static str => "Unknown Title",
    ALT_TITLE: &'static str => "Unknown Title",
    ARTIST: &'static str => "Unknown Artist",
    ALT_ARTIST: &'static str => "Unknown Artist",
    CREATOR: &'static str => "Unknown Creator",
    GENRE: &'static str => "Unknown Genre",
    SOURCE: &'static str => "Unknown Source",
    TAGS: Vec<String> => Vec::<String>::new(),

    BPM: &'static f32 => &0.0,
    DIFFICULTY_NAME: &'static str => "Unknown Difficulty",
    BG_PATH: &'static str => "Unknown Background Path",
    SONG_PATH: &'static str => "Unknown Song File Path",
    AUDIO_OFFSET: &'static i32 => &0,
    PREVIEW_TIME: &'static i32 => &0,
    OVERALL_DIFFICULTY: &'static f32 => &7.2,
    KEY_COUNT: &'static u8 => &4,
    
    RAW_NOTES: &'static str => "No Note Data",
    RAW_BPMS: &'static str => "No BPM Data",
    RAW_STOPS: &'static str => "No STOPS Data",
    RAW_SV: &'static str => "No SV Data",

    HITSOUND: [u8; 4] => [0, 0, 0, 0],
});

pub type Row = Vec<Key>;
pub type Measure = Vec<Row>;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimingChangeType {
    Bpm,
    Sv,
    Stop
}

#[allow(unused)]
#[derive(Debug)]
pub enum GameMode {
    Mania,
    Taiko,
    Catch,
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Mania => write!(f, "mania"),
            Self::Taiko => write!(f, "taiko"),
            Self::Catch => write!(f, "catch"),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType { 
    Empty,
    Normal,
    SliderStart,
    SliderEnd,
    Mine,
    Fake,
    Unknown,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    #[wasm_bindgen(getter_with_clone)]
    pub key_type: KeyType,
    #[wasm_bindgen(getter_with_clone)]
    pub slider_end_time: Option<i32>,
}

#[wasm_bindgen]
impl Key {
    #[wasm_bindgen]
    pub fn empty() -> Self {
        Self {
            key_type: KeyType::Empty,
            slider_end_time: None,
        }
    }

    #[wasm_bindgen]
    pub fn normal() -> Self {
        Self {
            key_type: KeyType::Normal,
            slider_end_time: None,
        }
    }

    #[wasm_bindgen]
    pub fn slider_start(value: Option<i32>) -> Self {
        Self {
            key_type: KeyType::SliderStart,
            slider_end_time: value,
        }
    }

    #[wasm_bindgen]
    pub fn slider_end() -> Self {
        Self {
            key_type: KeyType::SliderEnd,
            slider_end_time: None,
        }
    }

    #[wasm_bindgen]
    pub fn mine() -> Self {
        Self {
            key_type: KeyType::Mine,
            slider_end_time: None,
        }
    }

    #[wasm_bindgen]
    pub fn fake() -> Self {
        Self {
            key_type: KeyType::Fake,
            slider_end_time: None,
        }
    }

    #[wasm_bindgen]
    pub fn unknown() -> Self {
        Self {
            key_type: KeyType::Unknown,
            slider_end_time: None,
        }
    }

    #[wasm_bindgen]
    pub fn slider_end_time(&self) -> Option<i32> {
        self.slider_end_time
    }
}
