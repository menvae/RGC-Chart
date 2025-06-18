use crate::models::common::{TimingChangeType, Row, KeyType};

#[inline(always)]
pub fn to_millis(number: f32) -> f32 {
    number * 1000f32
}

#[inline(always)]
pub fn to_seconds(number: f32) -> f32 {
    number / 1000f32
}

// If you make fun of using a match statement here
// I will find where you live and personally come to your house and what will happen next will have you praying that you were never born into this world
pub fn merge_bpm_and_stops(
    bpms_beats: Vec<f32>,
    bpms_values: Vec<f32>,
    stops_beats: Vec<f32>,
    stops_values: Vec<f32>,
) -> (Vec<f32>, Vec<f32>, Vec<TimingChangeType>) {
    let mut beats = Vec::new();
    let mut values = Vec::new();
    let mut types = Vec::new();

    match (bpms_beats.is_empty(), stops_beats.is_empty()) {
        (true, true) => (),
        (true, false) => {
            for i in 0..stops_beats.len() {
                beats.push(stops_beats[i]);
                values.push(stops_values[i]);
                types.push(TimingChangeType::Stop);
            }
        },
        (false, true) => {
            for i in 0..bpms_beats.len() {
                beats.push(bpms_beats[i]);
                values.push(bpms_values[i]);
                types.push(TimingChangeType::Bpm);
            }
        },
        (false, false) => {
            let mut merged: Vec<((f32, f32), TimingChangeType)> =
                Vec::with_capacity(bpms_beats.len() + stops_beats.len());

            for i in 0..bpms_beats.len() {
                merged.push(((bpms_beats[i], bpms_values[i]), TimingChangeType::Bpm));
            }
            for i in 0..stops_beats.len() {
                merged.push(((stops_beats[i], stops_values[i]), TimingChangeType::Stop));
            }

            merged.sort_unstable_by(|a, b| {
                a.0.0
                    .partial_cmp(&b.0.0)
                    .expect("Comparison between NaN values when merging bpm and stops")
            });

            for ((beat, value), change_type) in merged {
                beats.push(beat);
                values.push(value);
                types.push(change_type);
            }
        }
    }

    (beats, values, types)
}

#[inline(always)]
pub fn find_sliderend_time(
    start_idx: usize,
    key_idx: usize,
    hitobjects: &[(&i32, &f32, &Vec<u8>, &Row)],
) -> i32 {
    if start_idx >= hitobjects.len() {
        return 0;
    }

    let start_time = hitobjects[start_idx].0;
    let slice = &hitobjects[start_idx + 1..];
    
    for (time, _, _, row) in slice {
        if row[key_idx].key_type == KeyType::SliderEnd {
            return **time
        }
    }
    
    *start_time
}