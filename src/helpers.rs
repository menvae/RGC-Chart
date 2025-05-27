use std::str::FromStr;
use crate::chart_classes::TimingChangeType;

#[inline(always)]
pub fn to_millis(number: f32) -> f32 {
    number * 1000f32
}

#[inline(always)]
pub fn to_seconds(number: f32) -> f32 {
    number / 1000f32
}

pub fn remove_comments(string: &str) -> String {
    let mut result = String::with_capacity(string.len());
    for line in string.lines() {
        let (content, _) = line.split_once("//").unwrap_or((line, ""));
        if content.chars().any(|c| !c.is_whitespace()) {
            result.push_str(content);
            result.push('\n');
        }
    }
    result.pop();
    result
}

#[inline]
pub fn parse_key_value(raw_str: &str) -> (&str, &str) {
    let colon_pos = raw_str.find(':').unwrap();
    (raw_str[..colon_pos].trim(), raw_str[colon_pos + 1..].trim())
}

#[inline(always)]
pub fn trim_split_iter<'a, I>(split_iter: I) -> Vec<&'a str>
where
    I: Iterator<Item = &'a str>,
{
    split_iter.map(|s| s.trim()).collect()
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
        (true, true) => (), // All vectors remain empty
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

pub trait StrDefaultExtension {
    fn or_default_empty(&self, default: &str) -> String;
}

impl StrDefaultExtension for str {
    fn or_default_empty(&self, default: &str) -> String {
        let trimmed = self.trim();
        if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed.to_string()
        }
    }
}

pub trait StrNumericDefaultExtension {
    fn or_default_empty_as<D: Default + ToString + FromStr>(&self, default: D) -> D;
}

impl StrNumericDefaultExtension for str {
    fn or_default_empty_as<D: Default + ToString + FromStr>(&self, default: D) -> D {
        let s = self.trim();
        if s.is_empty() {
            return default;
        }

        s.parse().unwrap_or_else(|_| {
            default.to_string().parse().unwrap_or_else(|_| {
                panic!(
                    "Failed to parse '{}' or default '{}' as requested numeric type",
                    s,
                    default.to_string()
                )
            })
        })
    }
}

#[inline(always)]
pub fn approx_eq(a: f32, b: f32, margin: f32) -> bool {
    (a - b).abs() <= margin
}

#[inline(always)]
pub fn thresholded_ceil(value: f32, threshold: f32) -> f32 {
    let fractional = value.fract();
    if fractional >= threshold {
        value.floor() + 1.0
    } else {
        value
    }
}

#[inline]
pub fn add_key_value_template(template: &mut String, key: &str, sep: &str, value: &str, end: &str) {
    template.reserve(key.len() + value.len() + 3);
    template.push_str(key);
    template.push_str(sep);
    template.push_str(value);
    template.push_str(end);
}