use crate::models;
use crate::models::common::{GameMode, KeyType};
use crate::utils::string::add_key_value_template;
use crate::utils::time::find_sliderend_time;
use crate::errors;

fn generate_timing_point(time: f32, bpm: f32) -> String {
    let time_str = time.to_string();
    let bpm_str = bpm.to_string();
    let mut result = String::with_capacity(time_str.len() + bpm_str.len() + 21);
    result.push_str("- StartTime: ");
    result.push_str(&time_str);
    result.push('\n');
    result.push_str("  Bpm: ");
    result.push_str(&bpm_str);
    result
}

fn generate_sv(time: f32, multiplier: f32) -> String {
    let time_str = time.to_string();
    let multiplier_str = multiplier.to_string();
    let mut result = String::with_capacity(time_str.len() + multiplier_str.len() + 28);
    result.push_str("- StartTime: ");
    result.push_str(&time_str);
    result.push('\n');
    result.push_str("  Multiplier: ");
    result.push_str(&multiplier_str);
    result
}

fn generate_hitobject(time: f32, slider_end_time: Option<f32>, column: usize) -> String {
    let is_slider = slider_end_time.is_some();
    let time_str = time.to_string();
    let column_str = (column + 1).to_string();
    let mut slider_end_time_str = String::new();

    if is_slider {
        slider_end_time_str = slider_end_time.unwrap().to_string();
    }
    
    let mut result = String::with_capacity(time_str.len() + column_str.len() + slider_end_time_str.len() + 38);
    result.push_str("- StartTime: ");
    result.push_str(&time_str);
    result.push('\n');
    result.push_str("  Lane: ");
    result.push_str(&column_str);
    result.push('\n');
    if is_slider {
        result.reserve(11);
        result.push_str("  EndTime: ");
        result.push_str(&slider_end_time_str);
        result.push('\n');
    }
    result.push_str("  KeySounds: []");
    result
}

pub(crate) fn to_qua(chart: &models::chart::Chart) -> Result<String, Box<dyn std::error::Error>> {
    let mut template = String::new();
    let key_count = chart.chartinfo.key_count;

    // metadata
    add_key_value_template(&mut template,
        "AudioFile", ": ", &chart.chartinfo.song_path, "\n");
    add_key_value_template(&mut template,
        "SongPreviewTime", ": ", &chart.chartinfo.preview_time.to_string(), "\n");
    add_key_value_template(&mut template,
        "BackgroundFile", ": ", &chart.chartinfo.bg_path, "\n");
    
    if key_count == 4 || key_count == 7 {
        add_key_value_template(&mut template,
        "Mode", ": ", &format!("Keys{}", key_count), "\n");
    }
    else if key_count == 8 {
        add_key_value_template(&mut template,
        "Mode", ": ", "Keys7", "\n");
    } else {
        return Err( Box::new(errors::WriteError::<GameMode>::InvalidKeyCount(key_count, "4k, 7k and 7k+1".to_string(), "Quaver".to_string()) ) );
    }
    
    add_key_value_template(&mut template,
        "Title", ": ", &chart.metadata.title.replace("\n", ""), "\n");
    add_key_value_template(&mut template,
        "Artist", ": ", &chart.metadata.artist, "\n");
    add_key_value_template(&mut template,
        "Source", ": ", &chart.metadata.source, "\n");
    add_key_value_template(&mut template,
        "Tags", ": ", &chart.metadata.tags.join(","), "\n");
    add_key_value_template(&mut template,
        "Creator", ": ", &chart.metadata.creator, "\n");
    add_key_value_template(&mut template,
        "DifficultyName", ": ", &chart.chartinfo.difficulty_name, "\n");
    template.push_str("BPMDoesNotAffectScrollVelocity: true\n");
    template.push_str("InitialScrollVelocity: 1\n");
    template.push_str("EditorLayers: []\n");
    template.push_str("CustomAudioSamples: []\n");
    template.push_str("SoundEffects: []\n");
    template.push_str("TimingPoints:");
    if chart.timing_points.is_bpms_empty() {
        template.push_str(" []\n");
    } else {
        template.push('\n');
        for (time, _, change) in chart.timing_points.bpm_changes_zipped() {
            template.push_str(&generate_timing_point(*time, change.value));
            template.push('\n');
        }
    }

    // process timing points
    template.push_str("SliderVelocities:");
    if chart.timing_points.is_sv_empty() {
        template.push_str(" []\n");
    } else {
        template.push('\n');
        for (time, _, change) in chart.timing_points.sv_changes_zipped() {
            template.push_str(&generate_sv(*time, change.value));
            template.push('\n');
        }
    }


    // process hitobjects
    let hitobjects: Vec<(&f32, &f32, &Vec<u8>, &Vec<KeyType>)> = chart.hitobjects.iter_zipped().collect();
    template.reserve(hitobjects.len() * key_count as usize);
    template.push_str("HitObjects:");
    if chart.timing_points.is_bpms_empty() {
        template.push_str(" []\n");
    } else {
        template.push('\n');
        for (row_idx, (time, _, _hitsounds, row)) in hitobjects.iter().enumerate() {
            for (i, key_type) in row.iter().enumerate() {
                match key_type {
                    KeyType::Normal => {
                        template.push_str(&generate_hitobject(**time, None, i));
                        template.push('\n');
                    },
                    KeyType::SliderStart => {
                        let slider_end_time = find_sliderend_time(row_idx, i, &hitobjects);
                        template.push_str(&generate_hitobject(**time, Some(slider_end_time), i));
                        template.push('\n');
                    },
                    _ => continue,
                }
            }
        }
    }

    Ok(template)
}