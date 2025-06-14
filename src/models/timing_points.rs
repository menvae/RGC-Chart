use crate::wasm_bindgen;
use crate::models::common::TimingChangeType;

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


// TODO: add wasm bindings for Timings
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

    pub fn is_bpms_empty(&self) -> bool {
        !self.change_types.iter()
            .any(|change_type| matches!(change_type, TimingChangeType::Bpm))
    }

    pub fn is_sv_empty(&self) -> bool {
        !self.change_types.iter()
            .any(|change_type| matches!(change_type, TimingChangeType::Sv))
    }
}