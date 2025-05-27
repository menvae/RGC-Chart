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
    AUDIO_OFFSET: &'static f32 => &0.0,
    PREVIEW_TIME: &'static f32 => &0.0,
    OVERALL_DIFFICULTY: &'static f32 => &7.2,
    KEY_COUNT: &'static u8 => &4,
    
    RAW_NOTES: &'static str => "No Note Data",
    RAW_BPMS: &'static str => "No BPM Data",
    RAW_STOPS: &'static str => "No STOPS Data",

    HITSOUND: [u8; 4] => [0, 0, 0, 0],
});

pub type Row = Vec<KeyType>;
pub type Measure = Vec<Row>;
pub type BpmsAndStops = (Vec<f32>, Vec<f32>, Vec<TimingChangeType>);

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

#[allow(unused)]
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
#[derive(Debug, Clone)]
pub struct Metadata {
    #[wasm_bindgen(getter_with_clone)]
    pub title: String,
    #[wasm_bindgen(getter_with_clone)]
    pub alt_title: String,
    #[wasm_bindgen(getter_with_clone)]
    pub artist: String,
    #[wasm_bindgen(getter_with_clone)]
    pub alt_artist: String,
    #[wasm_bindgen(getter_with_clone)]
    pub creator: String,
    #[wasm_bindgen(getter_with_clone)]
    pub genre: String,
    #[wasm_bindgen(getter_with_clone)]
    pub tags: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub source: String,
}

impl Metadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        alt_title: String,
        artist: String,
        alt_artist: String,
        creator: String,
        genre: String,
        source: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            title,
            alt_title,
            artist,
            alt_artist,
            creator,
            genre,
            source,
            tags,  
        }
    }

    pub fn empty() -> Self {
        Self {
            title: ChartDefaults::TITLE.to_string(),
            alt_title: ChartDefaults::ALT_TITLE.to_string(),
            artist: ChartDefaults::ARTIST.to_string(),
            alt_artist: ChartDefaults::ALT_ARTIST.to_string(),
            creator: ChartDefaults::CREATOR.to_string(),
            genre: ChartDefaults::GENRE.to_string(),
            source: ChartDefaults::SOURCE.to_string(),
            tags: Vec::with_capacity(5),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ChartInfo {
    #[wasm_bindgen(getter_with_clone)]
    pub difficulty_name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub bg_path: String,
    #[wasm_bindgen(getter_with_clone)]
    pub song_path: String,
    #[wasm_bindgen(getter_with_clone)]
    pub audio_offset: f32,
    #[wasm_bindgen(getter_with_clone)]
    pub preview_time: f32,
    #[wasm_bindgen(getter_with_clone)]
    pub key_count: u8,
    #[wasm_bindgen(getter_with_clone)]
    pub row_count: u32,
    #[wasm_bindgen(getter_with_clone)]
    pub object_count: u32,
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
impl ChartInfo {
    pub fn new(
        difficulty_name: String,
        bg_path: String,
        song_path: String,
        audio_offset: f32,
        preview_time: f32,
        key_count: u8,
        row_count: u32,
        object_count: u32,
    ) -> Self {
        Self {
            difficulty_name,
            bg_path,
            song_path,
            audio_offset,
            preview_time,
            key_count,
            row_count,
            object_count,
        }
    }

    pub fn empty() -> Self {
        Self {
            difficulty_name: String::with_capacity(20),
            bg_path: String::with_capacity(20),
            song_path: String::with_capacity(10),
            audio_offset: 0.0,
            preview_time: 0.0,
            key_count: 4,
            row_count: 0,
            object_count: 0,
        }
    }
}

#[derive(Debug)]
#[repr(C, align(8))]
pub struct TimingPointView<'a> {
    pub time: &'a f32,
    pub bpm: &'a f32,
    pub beat: &'a f32,
    pub multiplier: &'a f32,
    pub kiai: &'a bool,
    pub change_type: TimingChangeType,
}


// todo: add wasm bindings for Timings
#[wasm_bindgen]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TimingPoints {
    #[wasm_bindgen(skip)]
    pub times: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub bpms: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub beats: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub multipliers: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub kiais: Vec<bool>,
    #[wasm_bindgen(skip)]
    pub change_types: Vec<TimingChangeType>,
}

// #[wasm_bindgen]
// impl Timings {
//     #[wasm_bindgen(getter)]
//     pub fn kiais(&self) -> Vec<Boolean> {
//         self.kiais
//             .iter()
//             .map(|&b| Boolean::from(b))
//             .collect()
//     }
// }

impl TimingPoints {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            times: Vec::with_capacity(capacity),
            multipliers: Vec::with_capacity(capacity),
            bpms: Vec::with_capacity(capacity),
            beats: Vec::with_capacity(capacity),
            kiais: Vec::with_capacity(capacity),
            change_types: Vec::with_capacity(capacity),
        }
    }

    pub fn new(
        times: Vec<f32>,
        bpms: Vec<f32>,
        beats: Vec<f32>,
        multipliers: Vec<f32>,
        kiais: Vec<bool>,
        change_types: Vec<TimingChangeType>,
    ) -> Self {
        Self {
            times,
            bpms,
            beats,
            multipliers,
            kiais,
            change_types,
        }
    }

    pub fn add(
        &mut self,
        time: f32,
        multiplier: f32,
        bpm: f32,
        beat: f32,
        kiai: bool,
        change_type: TimingChangeType,
    ) {
        self.times.push(time);
        self.multipliers.push(multiplier);
        self.bpms.push(bpm);
        self.beats.push(beat);
        self.kiais.push(kiai);
        self.change_types.push(change_type);
    }

    /// time, multiplier, bpm, beat, is_kiai, change_type
    pub fn iter_zipped(
        &self,
    ) -> impl Iterator<Item = (&f32, &f32, &f32, &f32, &bool, &TimingChangeType)> {
        self.times
            .iter()
            .zip(self.multipliers.iter())
            .zip(self.bpms.iter())
            .zip(self.beats.iter())
            .zip(self.kiais.iter())
            .zip(self.change_types.iter())
            .map(|(((((time, multiplier), bpm), beat), kiai), change_type)| {
                (time, multiplier, bpm, beat, kiai, change_type)
            })
    }

    pub fn iter_views(&self) -> impl Iterator<Item = TimingPointView> + '_ {
        self.times
            .iter()
            .zip(self.multipliers.iter())
            .zip(self.bpms.iter())
            .zip(self.beats.iter())
            .zip(self.kiais.iter())
            .zip(self.change_types.iter())
            .map(|(((((time, multiplier), bpm), beat), kiai), change_type)| TimingPointView {
                time,
                multiplier,
                bpm,
                beat,
                kiai,
                change_type: *change_type,
            })
    }
    
    pub fn bpm_changes_views(&self) -> impl Iterator<Item = TimingPointView> + '_ {
        self.iter_views().filter(|v| matches!(v.change_type, TimingChangeType::Bpm))
    }
    
    pub fn sv_changes_views(&self) -> impl Iterator<Item = TimingPointView> + '_ {
        self.iter_views().filter(|v| matches!(v.change_type, TimingChangeType::Sv))
    }

    /// time, multiplier, bpm, beat, is_kiai, change_type
    pub fn bpm_changes_zipped(
        &self,
    ) -> impl Iterator<Item = (&f32, &f32, &f32, &f32, &bool, &TimingChangeType)> + '_ {
        self.iter_zipped()
            .filter(|(_, _, _, _, _, change_type)| matches!(change_type, TimingChangeType::Bpm))
    }
    
    /// time, multiplier, bpm, beat, is_kiai, change_type
    pub fn sv_changes_zipped(
        &self,
    ) -> impl Iterator<Item = (&f32, &f32, &f32, &f32, &bool, &TimingChangeType)> + '_ {
        self.iter_zipped()
            .filter(|(_, _, _, _, _, change_type)| matches!(change_type, TimingChangeType::Sv))
    }
}

#[derive(Debug)]
#[repr(C, align(64))]
pub struct HitObjectView<'a> {
    pub time: &'a f32,
    pub row: &'a [KeyType],
    pub beat: &'a f32,
    pub hitsound: &'a [u8],
    _pad: [u8; 20],
}

impl<'a> HitObjectView<'a> {
    pub fn new(
        time: &'a f32,
        beat: &'a f32,
        key: &'a [KeyType],
        hitsound: &'a [u8],
    ) -> Self {
        Self {
            time,
            beat,
            row: key,
            hitsound,
            _pad: [0; 20],
        }
    }
}

// todo: add wasm bindings for HitObject
// todo: add row and object count
#[wasm_bindgen]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HitObjects {
    #[wasm_bindgen(skip)]
    pub times: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub rows: Vec<Row>,
    #[wasm_bindgen(skip)]
    pub beats: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub hitsounds: Vec<Vec<u8>>,
}

impl HitObjects {

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            times: Vec::with_capacity(capacity),
            hitsounds: Vec::with_capacity(capacity),
            rows: Vec::with_capacity(capacity),
            beats: Vec::with_capacity(2_u32.pow(7) as usize)
        }
    }

    pub fn new(times: Vec<f32>, hitsounds: Vec<Vec<u8>>, rows: Vec<Row>, beats: Vec<f32>) -> Self {
        Self {
            times,
            hitsounds,
            rows,
            beats,
        }
    }

    #[inline]
    pub fn add_hitobject(&mut self, time: f32, beat: f32, hitsound: Vec<u8>, row: Row) {
        if row.iter().all(|&note| note == KeyType::Empty) { return; }
        self.times.push(time);
        self.hitsounds.push(hitsound);
        self.beats.push(beat);
        self.rows.push(row);
    }


    /// time, beat, hitsounds, row
    pub fn iter_zipped(&self) -> impl Iterator<Item = (&f32, &f32, &Vec<u8>, &Row)> {
        self.times
            .iter()
            .zip(self.beats.iter())
            .zip(self.hitsounds.iter())
            .zip(self.rows.iter())
            .map(|(((time, beat), hitsound), key)| (time, beat, hitsound, key))
    }

    pub fn iter_views(&self) -> impl Iterator<Item = HitObjectView> {
        self.times
            .iter()
            .zip(self.hitsounds.iter())
            .zip(self.rows.iter())
            .zip(self.beats.iter())
            .map(|(((time, hitsound), key), beat)| {
                HitObjectView::new(time, beat, key, hitsound)
            })
    }
}

// todo: maybe don't use getter_with_clone?
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Chart {
    #[wasm_bindgen(getter_with_clone)]
    pub metadata: Metadata,
    #[wasm_bindgen(getter_with_clone)]
    pub chartinfo: ChartInfo,
    #[wasm_bindgen(getter_with_clone)]
    pub timing_points: TimingPoints,
    #[wasm_bindgen(getter_with_clone)]
    pub hitobjects: HitObjects
}

impl Chart {
    pub(crate) fn new(metadata: Metadata, chartinfo: ChartInfo, timings: TimingPoints, hitobjects: HitObjects) -> Self {
        Self {
            metadata,
            chartinfo,
            timing_points: timings,
            hitobjects,
        }
    }
}
