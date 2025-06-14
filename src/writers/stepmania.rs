use crate::models;
use crate::models::common::{
    ChartDefaults,
    KeyType,
    Measure,
};
use crate::utils::helpers::{
    add_key_value_template,
    approx_eq,
    to_seconds,
};
use crate::utils::rhythm::{
    MeasureRange,
    snap_to_nearest_note_type_normed,
};
#[allow(unused)]
use crate::errors;

#[inline]
fn sm_row_to_str(row: &[KeyType]) -> String {
    let mut result = String::with_capacity(row.len());
    for note in row {
        result.push(match note {
            KeyType::Empty => '0',
            KeyType::Normal => '1',
            KeyType::SliderStart => '2',
            KeyType::SliderEnd => '3',
            KeyType::Mine => 'M',
            KeyType::Fake => 'F',
            KeyType::Unknown => '0',
        });
    }
    result
}

#[inline]
fn pad_measure(rows: &models::hitobjects::HitObjects, range: &MeasureRange) -> Measure {
    let key_count = rows.iter_zipped().next().map_or(0, |row| row.3.len());

    if range.is_empty() {
        return vec![vec![KeyType::Empty; key_count]; 4];
    }

    let measure: Vec<_> = rows.iter_zipped()
        .skip(range.0)
        .take(range.1 - range.0)
        .collect();

    if measure.is_empty() {
        return vec![vec![KeyType::Empty; key_count]; 4];
    }
    

    let measure_start_beat = measure[0].1;
    let normalized_measure: Vec<_> = measure.iter()
        .map(|row| (row.0, row.1 - measure_start_beat, row.2, row.3))
        .collect();

    let mut minimal_beat_diff = 4.0;
    for i in 1..normalized_measure.len() {
        let diff = normalized_measure[i].1 - normalized_measure[i-1].1;
        if diff < minimal_beat_diff && diff > 1e-5 {
            minimal_beat_diff = diff;
        }
    }

    let snapped_beat_diff = snap_to_nearest_note_type_normed(minimal_beat_diff);
    let row_count = (4.0 / snapped_beat_diff).round() as usize;
    
    let expected_beats: Vec<f32> = (0..row_count)
        .map(|k| k as f32 * snapped_beat_diff)
        .collect();

    let mut padded_measure = Vec::with_capacity(row_count);
    let mut measure_iter = normalized_measure.iter().peekable();

    for &expected_beat in &expected_beats {
        if let Some(row) = measure_iter.peek() {
            if approx_eq(row.1, expected_beat, 0.15) {
                padded_measure.push(row.3.clone());
                measure_iter.next();
                continue;
            }
        }
        padded_measure.push(vec![KeyType::Empty; key_count]);
    }

    padded_measure
}

pub(crate) fn to_sm(chart: &models::chart::Chart) -> Result<String, Box<dyn std::error::Error>> {
    let mut template = String::new();
    let mut bpm_template = String::new();
    let mut notes_template = String::new();

    let mut prev_beat = 0.0;
    let mut prev_measure = 0;
    let mut prev_measure_beat = 0.0;
    let mut current_measure_index = 0;

    let scale_factor = 24.0;
    let beats_per_measure = 4.0;
    let beats_per_measure_scaled = scale_factor * beats_per_measure;

    let mut measure_indices: Vec<MeasureRange>  = Vec::with_capacity(85);
    let mut padded_measures: Vec<Measure> = Vec::with_capacity(chart.hitobjects.times.len() * 2);
    let bpms: Vec<_> = chart.timing_points.bpm_changes_views().collect();

    // get measures
    for (row_index, beat) in chart.hitobjects.beats.iter().enumerate() {
        
        let beat_scaled = (beat * scale_factor).round();
        let measure = (beat_scaled / beats_per_measure_scaled) as u32;

        if measure != prev_measure {
            measure_indices.push(MeasureRange(current_measure_index, row_index, false));

            if beat - prev_measure_beat > 5.0 {
                let empty_measure_count = ((beat - prev_beat) / 4.0).floor() as usize;
                
                for _ in 0..empty_measure_count {
                    measure_indices.push(MeasureRange(current_measure_index, row_index, true));
                    current_measure_index = row_index; 
                }
            }
            
            current_measure_index = row_index;
            prev_measure = measure;
            prev_measure_beat = *beat;
        }
        prev_beat = *beat;
    }
    measure_indices.push(MeasureRange(current_measure_index, chart.hitobjects.rows.len(), false));

    for measure_range in measure_indices {
        padded_measures.push(pad_measure(&chart.hitobjects, &measure_range));
    }

    // process bpms
    let last_bpm_beat = bpms.last().unwrap().beat;
    for bpm in bpms {
        if bpm.beat < last_bpm_beat {
            add_key_value_template(&mut bpm_template, &bpm.beat.to_string(), "=", &bpm.bpm.to_string(), ",\n");
        } else {
            add_key_value_template(&mut bpm_template, &bpm.beat.to_string(), "=", &bpm.bpm.to_string(), "\n");
        }
    }

    // process notes
    let last_measure_index = padded_measures.len() - 1;
    notes_template.push('\n');
    notes_template.push_str("   ");
    notes_template.push_str("dance-single:\n"); // TODO: support for multikey later
    notes_template.push_str("   ");
    notes_template.push_str(&chart.metadata.creator);
    notes_template.push_str(":\n");
    notes_template.push_str("   ");
    notes_template.push_str("Edit"); // TODO: maybe process difficulty value and choose an sm difficulty??
    notes_template.push_str(":\n");
    notes_template.push_str("   ");
    notes_template.push_str("1:\n");
    notes_template.push_str("   ");
    notes_template.push_str("0.000,0.000,0.000,0.000,0.000:\n");
    for (measure_index, measure) in padded_measures.iter().enumerate() {
        notes_template.push_str("// Measure ");
        notes_template.push_str(&(measure_index + 1).to_string());
        notes_template.push('\n');
        for row in measure {
            notes_template.push_str(&sm_row_to_str(row));
            notes_template.push('\n');
        }
        if measure_index != last_measure_index { notes_template.push_str(", "); }
    }

    // process template
    add_key_value_template(&mut template,
        "#TITLE", ":", &chart.metadata.title, ";\n");
    let subtitle = if chart.metadata.source == ChartDefaults::SOURCE {
        ""
    } else {
        &chart.metadata.source
    };
    add_key_value_template(&mut template,
        "#SUBTITLE", ":", subtitle, ";\n");
    add_key_value_template(&mut template,
        "#ARTIST",":", &chart.metadata.artist, ";\n");
    add_key_value_template(&mut template,
        "#TITLETRANSLIT",":", &chart.metadata.alt_title, ";\n");   
    add_key_value_template(&mut template,
        "#SUBTITLETRANSLIT",":", "", ";\n");  
    add_key_value_template(&mut template,
        "#ARTISTTRANSLIT",":", &chart.metadata.alt_artist, ";\n");
    add_key_value_template(&mut template,
        "#GENRE",":", &chart.metadata.genre, ";\n");
    add_key_value_template(&mut template,
        "#CREDIT",":", &chart.metadata.creator, ";\n");
    add_key_value_template(&mut template,
        "#BANNER",":", &chart.chartinfo.bg_path, ";\n");
    add_key_value_template(&mut template,
        "#BACKGROUND",":", &chart.chartinfo.bg_path, ";\n");
    add_key_value_template(&mut template,
        "#LYRICSPATH",":", "", ";\n");
    add_key_value_template(&mut template,
        "#CDTITLE",":", "", ";\n");
    add_key_value_template(&mut template,
        "#MUSIC",":", &chart.chartinfo.song_path, ";\n");
    add_key_value_template(&mut template,
        "#OFFSET",":",  &to_seconds(-chart.chartinfo.audio_offset).to_string(), ";\n");
    add_key_value_template(&mut template,
        "#SAMPLESTART",":",  &to_seconds(chart.chartinfo.preview_time).to_string(), ";\n");
    add_key_value_template(&mut template,
        "#SAMPLELENGTH",":",  "12.000", ";\n"); // TODO: maybe add chart length in chart info
    add_key_value_template(&mut template,
        "#SELECTABLE",":", "YES", ";\n");
    add_key_value_template(&mut template,
        "#BPMS",":", &bpm_template, ";\n");
    add_key_value_template(&mut template,
        "#STOPS",":", "", ";\n");
    add_key_value_template(&mut template,
        "#BGCHANGES",":", "", ";\n");
    add_key_value_template(&mut template,
        "#KEYSOUNDS",":", "", ";\n");
    add_key_value_template(&mut template,
        "#NOTES", ":", &notes_template, ";\n");

    Ok(template)
}

#[allow(unused)]
pub(crate) fn to_sma(chart: &models::chart::Chart) -> Result<String, Box<dyn std::error::Error>> {
    unimplemented!();
}

#[allow(unused)]
pub(crate) fn to_ssc(chart: &models::chart::Chart) -> Result<String, Box<dyn std::error::Error>> {
    unimplemented!();
}
