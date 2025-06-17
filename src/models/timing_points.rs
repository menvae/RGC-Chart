use crate::wasm_bindgen;
use crate::models::common::TimingChangeType;


#[derive(Debug)]
#[repr(C, align(8))]
pub struct TimingPointView<'a> {
    pub time: &'a i32,
    pub beat: &'a f32,
    pub change_type: TimingChangeType,
    pub value: &'a f32,
}

#[derive(Debug, Clone)]
pub struct TimingChange {
    pub change_type: TimingChangeType,
    pub value: f32,
}

// TODO: add wasm bindings for Timings
#[wasm_bindgen]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TimingPoints {
    #[wasm_bindgen(skip)]
    pub times: Vec<i32>,
    #[wasm_bindgen(skip)]
    pub beats: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub changes: Vec<TimingChange>,
}

impl TimingPoints {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            times: Vec::with_capacity(capacity),
            beats: Vec::with_capacity(capacity),
            changes: Vec::with_capacity(capacity),
        }
    }

    pub fn new(
        times: Vec<i32>,
        beats: Vec<f32>,
        changes: Vec<TimingChange>,
    ) -> Self {
        Self {
            times,
            beats,
            changes,
        }
    }

    pub fn add(
        &mut self,
        time: i32,
        beat: f32,
        change: TimingChange,
    ) {
        self.times.push(time);
        self.beats.push(beat);
        self.changes.push(change);
    }

    /// time, beat, change
    pub fn iter_zipped(
        &self,
    ) -> impl Iterator<Item = (&i32, &f32, &TimingChange)> {
        self.times
            .iter()
            .zip(self.beats.iter())
            .zip(self.changes.iter())
            .map(|((time, beat), change)| (time, beat, change))
    }

    pub fn iter_views(&self) -> impl Iterator<Item = TimingPointView> + '_ {
        self.times
            .iter()
            .zip(self.beats.iter())
            .zip(self.changes.iter())
            .map(|((time, beat), change)| TimingPointView {
                time,
                beat,
                change_type: change.change_type,
                value: &change.value,
            })
    }
    
    pub fn bpm_changes_views(&self) -> impl Iterator<Item = TimingPointView> + '_ {
        self.iter_views().filter(|v| matches!(v.change_type, TimingChangeType::Bpm))
    }
    
    pub fn sv_changes_views(&self) -> impl Iterator<Item = TimingPointView> + '_ {
        self.iter_views().filter(|v| matches!(v.change_type, TimingChangeType::Sv))
    }

    /// time, beat, change
    pub fn bpm_changes_zipped(
        &self,
    ) -> impl Iterator<Item = (&i32, &f32, &TimingChange)> + '_ {
        self.iter_zipped()
            .filter(|(_, _, change)| matches!(change.change_type, TimingChangeType::Bpm))
    }
    
    /// time, beat, change
    pub fn sv_changes_zipped(
        &self,
    ) -> impl Iterator<Item = (&i32, &f32, &TimingChange)> + '_ {
        self.iter_zipped()
            .filter(|(_, _, change)| matches!(change.change_type, TimingChangeType::Sv))
    }

    pub fn is_bpms_empty(&self) -> bool {
        !self.changes.iter()
            .any(|change| matches!(change.change_type, TimingChangeType::Bpm))
    }

    pub fn is_sv_empty(&self) -> bool {
        !self.changes.iter()
            .any(|change| matches!(change.change_type, TimingChangeType::Sv))
    }

    pub fn bpms(&self) -> Vec<f32> {
        self.changes.iter()
            .filter(|change| matches!(change.change_type, TimingChangeType::Bpm))
            .map(|change| change.value)
            .collect()
    }

    pub fn sv(&self) -> Vec<f32> {
        self.changes.iter()
            .filter(|change| matches!(change.change_type, TimingChangeType::Sv))
            .map(|change| change.value)
            .collect()
    }
}
