use crate::helpers::{
    to_millis,
    to_seconds,
    remove_comments,
    trim_split_iter,
    StrDefaultExtension,
    StrNumericDefaultExtension,
    merge_bpm_and_stops,
    add_key_value_template,
    approx_eq
};
use crate::chart_classes::{
    self,
    BpmsAndStops,
    ChartDefaults,
    GameMode,
    KeyType,
    Measure,
    TimingChangeType,
};
use crate::rhythm::{
    calculate_beat_from_time,
    calculate_time_from_beat,
    MeasureRange,
    snap_to_nearest_note_type_normed
};

use crate::errors;

pub fn parse_beats(raw: &str) -> (Vec<f32>, Vec<f32>) {
    // lazy
    match raw {
        "No BPM Data" => return (vec![0.0], vec![*ChartDefaults::BPM]),
        "No STOPS Data" => return (vec![], vec![]),
        _ => {}
    }

    let mut beats = Vec::new();
    let mut bpms = Vec::new();

    raw.split(',')
        .filter_map(|beat_bpm_str| {
            let mut beat_bpm = beat_bpm_str.trim().split('=');

            if let (Some(beat_str), Some(bpm_str)) = (beat_bpm.next(), beat_bpm.next()) {
                if let (Ok(beat), Ok(bpm)) = (beat_str.parse::<f32>(), bpm_str.parse::<f32>()) {
                    debug_assert_ne!(bpm, 0.0);
                    return Some((beat, bpm));
                }
            }
            None
        })
        .for_each(|(beat, bpm)| {
            beats.push(beat);
            bpms.push(bpm);
        });

    (beats, bpms)
}


pub fn parse_keys_in_row(row: &str) -> Vec<chart_classes::KeyType> {

    let mut result: Vec<chart_classes::KeyType> = Vec::with_capacity(row.len());

    for c in row.chars() {
        result.push(get_sm_note_type(c));
    }

    result
}

#[inline]
pub(crate) const fn get_sm_note_type(note: char) -> KeyType {
    match note {
        '0' => KeyType::Empty,
        '1' => KeyType::Normal,
        '2' => KeyType::SliderStart,
        '3' => KeyType::SliderEnd,
        '4' => KeyType::SliderStart,
        'M' => KeyType::Mine,
        'F' => KeyType::Fake,
        _ => KeyType::Unknown,
    }
}

#[inline]
pub(crate) fn sm_row_to_str(row: &[KeyType]) -> String {
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
pub fn pad_measure(rows: &chart_classes::HitObjects, range: &MeasureRange) -> Measure {
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

pub fn process_headers<F>(raw_chart: &str, mut lambda: F)
where
    F: FnMut(&str, &str),
{
    for pair in raw_chart.split(';') {
        if let Some(colon_index) = pair.find(":") {
            let header = pair[..colon_index].trim();
            let content = pair[colon_index + 1..].trim();

            if let Some(second_colon_index) = content.find(":"){
              let first_content = content[..second_colon_index].trim();
              let second_content = content[second_colon_index + 1..].trim();
              let full_content = format!("{}:{}",first_content,second_content);
              lambda(header, full_content.as_str());
            } else {
              lambda(header, content);
            }
        }
    }
}

pub fn process_timing_points(bpms_and_stops: &BpmsAndStops, bpms_only: (Vec<f32>, Vec<f32>), start_time: f32) -> chart_classes::TimingPoints {
    use chart_classes::TimingPoints;
    let mut timing_points = TimingPoints::with_capacity(64);

    let (beats, bpms_and_durations, change_types) = bpms_and_stops;
    let (bpms_only_beats, bpms_only_values) = bpms_only;

    let mut prev_bpm = bpms_and_durations[0];

    for i in 0..beats.len() {
        let current_beat = beats[i];
        let bpm_or_duration = bpms_and_durations[i];

        let insert_time = calculate_time_from_beat(
            current_beat,
            start_time,
            (beats, bpms_and_durations, change_types)
        );
        
        match change_types[i] {
            TimingChangeType::Bpm => {
                timing_points.add(
                    insert_time,
                    1.0,
                    bpm_or_duration,
                    current_beat,
                    false,
                    TimingChangeType::Bpm
                );
                prev_bpm = bpm_or_duration;
            },
            TimingChangeType::Stop => {
                timing_points.add(
                    insert_time,
                    0.0, 
                    prev_bpm,
                    current_beat,
                    false,
                    TimingChangeType::Sv
                );

                let stop_end_time = insert_time + bpm_or_duration;
                
                let stop_end_beat = calculate_beat_from_time(
                    stop_end_time,
                    start_time,
                    (&bpms_only_beats, &bpms_only_values)
                );

                let stop_time = insert_time + to_millis(bpm_or_duration);

                timing_points.add(
                    stop_time,
                    1.0, 
                    prev_bpm,
                    stop_end_beat,
                    false,
                    TimingChangeType::Sv
                );
            },
            _ => {}
        }
    }

    timing_points
}

pub fn process_notes(raw_note_data: &str, chartinfo: &mut chart_classes::ChartInfo, bpms_and_stops: &BpmsAndStops) ->chart_classes::HitObjects {
    use crate::chart_classes::HitObjects;

    if raw_note_data.contains("No Note Data") { return HitObjects::with_capacity(2048) }

    let mut hitobjects = HitObjects::with_capacity(2048);
    let (beats, bpms_and_durations, change_types) = bpms_and_stops;
    
    let start_time = chartinfo.audio_offset;
    let separated_note_data: Vec<&str> = trim_split_iter(raw_note_data.split(":"));

    let difficulty_name = separated_note_data[2];
    chartinfo.difficulty_name = difficulty_name.or_default_empty(ChartDefaults::DIFFICULTY_NAME);

    // todo: make error for stepmania if converting from keys other than 4
    let key_count = 4; // todo: change this later if gonna make this function generic to support Beatmania

    let raw_notes = separated_note_data.last().unwrap_or(&"Failed to get raw notes in notes section");
    let measures: Vec<&str> = raw_notes.split(",").collect();

    let mut measure_beat_count: f32 = 0.0;
    let mut total_row_count = 0;
    let mut total_object_count = 0;

    for measure in measures {
        let trimmed_measure = measure.trim();
        let rows: Vec<_> = trimmed_measure.split('\n').collect();
        let row_count = rows.len();
        total_row_count += row_count;
        let beat_time_per_row = 4.0 / row_count as f32;
    
        for (row_index, row) in rows.into_iter().enumerate() {
            let row_beat = measure_beat_count + row_index as f32 * beat_time_per_row;
            
            let row_time = calculate_time_from_beat(
                row_beat,
                start_time,
                (beats, bpms_and_durations, change_types)
            );
    
            let keys = parse_keys_in_row(row);
            for key in &keys {
                if *key != KeyType::Empty {
                    total_object_count += 1;
                }
            }
            
            hitobjects.add_hitobject(
                row_time.round(),
                row_beat,
                vec![0; key_count],
                keys
            );
        }
    
        measure_beat_count += 4.0;
    }

    chartinfo.row_count = total_row_count as u32;
    chartinfo.object_count = total_object_count;

    hitobjects
}

pub(crate) fn parse_sm(raw_chart: &str) -> Result<chart_classes::Chart, Box<dyn std::error::Error>>  {
    use self::chart_classes::{Metadata, ChartInfo};

    let uncommented_chart = remove_comments(raw_chart);

    if uncommented_chart.trim().is_empty() {
        return Err(Box::new(errors::ParseError::<GameMode>::EmptyChartData));
    }

    let mut metadata = Metadata::empty();
    let mut chartinfo = ChartInfo::empty();

    let mut bpms: (std::vec::Vec<f32>, std::vec::Vec<f32>) = (vec![0.0], vec![0.0]);
    let mut raw_bpms = ChartDefaults::RAW_BPMS.to_string();
    let mut stops = (vec![], vec![]);
    let mut raw_stops = ChartDefaults::RAW_STOPS.to_string();
    let mut raw_notes = ChartDefaults::RAW_NOTES.to_string();

    

    process_headers(&uncommented_chart, |header, content| {
        match header {
            "#TITLE" => metadata.title = content.or_default_empty(ChartDefaults::TITLE),
            "#ARTIST" => metadata.artist = content.or_default_empty(ChartDefaults::ARTIST),
            "#SUBTITLE" => metadata.source = content.or_default_empty(ChartDefaults::SOURCE),
            "#TITLETRANSLIT" => metadata.alt_title = content.or_default_empty(ChartDefaults::ALT_TITLE),
            "#ARTISTTRANSLIT" => metadata.alt_artist = content.or_default_empty(ChartDefaults::ALT_ARTIST),
            "#SUBTITLETRANSLIT" => {},
            "#GENRE" => metadata.genre = content.or_default_empty(ChartDefaults::GENRE),
            "#CREDIT" => metadata.creator = content.or_default_empty(ChartDefaults::CREATOR),
            "#BACKGROUND"=> chartinfo.bg_path = content.or_default_empty(ChartDefaults::BG_PATH),
            "#MUSIC" => chartinfo.song_path = content.or_default_empty(ChartDefaults::SONG_PATH),
            "#OFFSET" => chartinfo.audio_offset = -to_millis(content.or_default_empty_as::<f32>(*ChartDefaults::AUDIO_OFFSET)),
            "#SAMPLESTART" => chartinfo.preview_time = to_millis(content.or_default_empty_as::<f32>(*ChartDefaults::PREVIEW_TIME)),
            "#BPMS" => {
                raw_bpms = content.or_default_empty(ChartDefaults::RAW_BPMS);
                bpms = parse_beats(&raw_bpms);
            },
            "#STOPS" => {
                raw_stops = content.or_default_empty(ChartDefaults::RAW_STOPS);
                stops = parse_beats(&raw_stops);
            },
            "#NOTES" => {
                raw_notes = content.or_default_empty(ChartDefaults::RAW_NOTES)
            },
            _ => {},
        }
    });
    
    let bpms_only = bpms.clone();
    let bpms_and_stops = merge_bpm_and_stops(bpms.0, bpms.1, stops.0, stops.1);

    let timing_points = process_timing_points(&bpms_and_stops, bpms_only, chartinfo.audio_offset);

    // chartinfo is passed as a mutable ref since it contains the start offset and we need to update it with the difficulty name from the note data.
    let hitobjects = process_notes(&raw_notes, &mut chartinfo, &bpms_and_stops);

    Ok(chart_classes::Chart::new(metadata, chartinfo, timing_points, hitobjects))
}

#[allow(unused)]
pub(crate) fn parse_sma(raw_chart: &str) -> Result<chart_classes::Chart, Box<dyn std::error::Error>>  {
    unimplemented!();
}

#[allow(unused)]
pub(crate) fn parse_ssc(raw_chart: &str) -> Result<chart_classes::Chart, Box<dyn std::error::Error>>  {
    unimplemented!();
}

pub(crate) fn convert_to_sm(chart: &chart_classes::Chart) -> Result<String, Box<dyn std::error::Error>> {

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
    notes_template.push_str("dance-single:\n"); // todo: support for multikey later
    notes_template.push_str("   ");
    notes_template.push_str(&chart.metadata.creator);
    notes_template.push_str(":\n");
    notes_template.push_str("   ");
    notes_template.push_str("Edit"); // todo: maybe process difficulty value and choose an sm difficulty??
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
        "#SAMPLELENGTH",":",  "12.000", ";\n"); // todo: maybe add chart length in chart info
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
pub(crate) fn convert_to_sma(chart: &chart_classes::Chart) -> Result<String, Box<dyn std::error::Error>> {
    unimplemented!();
}

#[allow(unused)]
pub(crate) fn convert_to_ssc(chart: &chart_classes::Chart) -> Result<String, Box<dyn std::error::Error>> {
    unimplemented!();
}
