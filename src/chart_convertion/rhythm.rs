use crate::helpers::{thresholded_ceil, to_millis};
use crate::chart_classes::TimingChangeType;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeasureRange(pub usize, pub usize, pub bool);
impl MeasureRange {
    #[inline(always)]
    pub fn start(&self) -> usize {
        self.0
    }

    #[inline(always)]
    pub fn end(&self) -> usize {
        self.1
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.2
    }
}

pub const BEAT_DENOMS: [usize; 9] = [4, 8, 12, 16, 24, 32, 48, 64, 192];
pub const NOTE_TYPES: [f32; 9] = [
        1.0,    // 4th (4/4)
        0.5,     // 8th (4/8)
        0.33333334, // 12th (4/12)
        0.25,    // 16th (4/16)
        0.16666667, // 24th (4/24)
        0.125,    // 32nd (4/32)
        0.083333336, // 48th (4/48)
        0.0625,   // 64th (4/64)
        0.020833334, // 192nd (4/192)
    ];

pub fn snap_to_nearest_note_type_normed(beat: f32) -> f32 {
    let mut min_diff  = f32::MAX;
    let mut nearest_note_tyoe  = NOTE_TYPES[0];
    
    let candidates = [
        (NOTE_TYPES[0], (NOTE_TYPES[0] - beat).abs()),
        (NOTE_TYPES[1], (NOTE_TYPES[1] - beat).abs()),
        (NOTE_TYPES[2], (NOTE_TYPES[2] - beat).abs()),
        (NOTE_TYPES[3], (NOTE_TYPES[3] - beat).abs()),
        (NOTE_TYPES[4], (NOTE_TYPES[4] - beat).abs()),
        (NOTE_TYPES[5], (NOTE_TYPES[5] - beat).abs()),
        (NOTE_TYPES[6], (NOTE_TYPES[6] - beat).abs()),
        (NOTE_TYPES[7], (NOTE_TYPES[7] - beat).abs()),
        (NOTE_TYPES[8], (NOTE_TYPES[8] - beat).abs()),
    ];
    
    for (note_type, diff) in candidates {
        if diff < min_diff {
            min_diff = diff;
            nearest_note_tyoe = note_type;
        }
    }
    
    nearest_note_tyoe
}

pub fn calculate_beat_from_time(
    time: f32,
    start_time: f32,
    timing_points: (&[f32], &[f32]), // (bpm_times, bpms)
) -> f32 {
    let (bpm_times, bpms) = timing_points;

    if bpm_times.is_empty() || bpms.is_empty() || bpm_times.len() != bpms.len() {
        return -1.0;
    }

    if time < start_time {
        return 0.0;
    }

    let initial_pos = bpm_times.partition_point(|&t| t <= start_time).saturating_sub(1);
    let initial_bpm = if initial_pos < bpm_times.len() && bpm_times[initial_pos] <= start_time {
        bpms[initial_pos]
    } else {
        0.0
    };

    let start_idx = bpm_times.partition_point(|&t| t <= start_time);
    let end_idx = bpm_times.partition_point(|&t| t <= time);

    let mut total_beats = 0.0;
    let mut prev_time = start_time;
    let mut current_bpm = initial_bpm;

    for i in start_idx..end_idx {
        let bpm_change_time = bpm_times[i];
        let new_bpm = bpms[i];
        let segment_duration = bpm_change_time - prev_time;
        total_beats += segment_duration * current_bpm / 60_000.0;
        prev_time = bpm_change_time;
        current_bpm = new_bpm;
    }

    let current_bpm_segment = time - prev_time;
    total_beats += current_bpm_segment * current_bpm / 60_000.0;

    thresholded_ceil(total_beats, 0.95) // I hate floats
}

pub fn calculate_time_from_beat(
    beat: f32,
    start_time: f32,
    timing_points: (&[f32], &[f32], &[TimingChangeType]), // (bpm_beats, bpms/durations, timing_change_types)
) -> f32 {
    let (bpm_beats, bpm_or_duration, change_types) = timing_points;

    if bpm_beats.is_empty() || bpm_or_duration.is_empty() || bpm_beats.len() != bpm_or_duration.len() {
        return -1.0;
    }

    if beat < 0.0 {
        return start_time;
    }

    let initial_pos = bpm_beats.partition_point(|&b| b <= 0.0).saturating_sub(1);
    let initial_bpm = if initial_pos < bpm_beats.len() && bpm_beats[initial_pos] <= 0.0 {
        match change_types[initial_pos] {
            TimingChangeType::Bpm => bpm_or_duration[initial_pos],
            _ => 0.0,
        }
    } else {
        0.0
    };

    let start_idx = bpm_beats.partition_point(|&b| b <= 0.0);
    let end_idx = bpm_beats.partition_point(|&b| b <= beat);

    let mut total_time = start_time;
    let mut prev_beat = 0.0;
    let mut current_bpm = initial_bpm;

    for i in start_idx..end_idx {
        let change_beat = bpm_beats[i];
        let segment_beats = change_beat - prev_beat;
        
        if current_bpm != 0.0 {
            total_time += segment_beats * (60000.0 / current_bpm);
        }
        
        prev_beat = change_beat;
        
        match change_types[i] {
            TimingChangeType::Bpm => {
                current_bpm = bpm_or_duration[i];
            }
            TimingChangeType::Stop => {
                total_time += to_millis(bpm_or_duration[i]);
            }
            _ => {}
        }
    }

    let current_segment_beats = beat - prev_beat;
    if current_bpm != 0.0 {
        total_time += current_segment_beats * (60000.0 / current_bpm);
    }

    total_time
}
