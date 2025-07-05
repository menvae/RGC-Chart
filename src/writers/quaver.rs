use crate::models;
use crate::models::sound::KeySound;
use models::sound::{KeySoundRow, HitSoundType};
use crate::models::common::{GameMode, KeyType, Row};
use crate::utils::string::{add_key_value_template, add_key_value_template_escaped};
use crate::utils::time::find_sliderend_time;
use crate::errors;

#[inline(always)]
fn generate_timing_point(time: i32, bpm: f32) -> String {
    format!("- StartTime: {}\n  Bpm: {}", time, bpm)
}

#[inline(always)]
fn generate_sv(time: i32, multiplier: f32) -> String {
    format!("- StartTime: {}\n  Multiplier: {}", time, multiplier)
}

#[inline(always)]
fn generate_soundeffect(time: i32, sample_index: usize, volume: u8) -> String {
    if volume >= 100 {
        format!("- StartTime: {}\n  Sample: {}", time, sample_index+1)
    } else {
        format!("- StartTime: {}\n  Sample: {}\n  Volume: {}", time, sample_index+1, volume)
    }
    
}

#[inline(always)]
fn generate_hitobject(time: i32, slider_end_time: Option<i32>, column: usize, keysound: KeySound) -> String {
    let lane = column + 1;
    let hitsound = if keysound.hitsound_type == HitSoundType::Normal {
        ""
    } else {
        &("  HitSound: ".to_string() + match keysound.hitsound_type {
            HitSoundType::Clap => "Clap",
            HitSoundType::Whistle => "Whistle",
            HitSoundType::Finish => "Finish", 
            _ => "Clap",
        } + "\n")
    };

    let keysounds = if keysound.has_custom {
        let sound_sample = keysound.sample.unwrap() + 1;
        if keysound.volume >= 100 {
            &format!("\n  - Sample: {}", sound_sample)
        } else {
            &format!("\n  - Sample: {}\n    Volume: {}", sound_sample, keysound.volume)
        }
    } else {
        " []"
    };

    match slider_end_time {
        Some(end_time) => format!(
            "- StartTime: {}\n  Lane: {}\n  EndTime: {}\n{}  KeySounds:{}",
            time, lane, end_time, hitsound, keysounds
        ),
        None => format!(
            "- StartTime: {}\n  Lane: {}\n{}  KeySounds:{}",
            time, lane, hitsound, keysounds
        ),
    }
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
    add_key_value_template_escaped(&mut template,
        "DifficultyName", ": ", &chart.chartinfo.difficulty_name, "\n");
    template.push_str("BPMDoesNotAffectScrollVelocity: true\n");
    template.push_str("InitialScrollVelocity: 1\n");
    template.push_str("EditorLayers: []\n");


    // process custom audio samples
    template.push_str("CustomAudioSamples:");
    match &chart.soundbank {
        Some(soundbank) => {
            let soundsamples = soundbank.get_sample_paths();
            if soundsamples.is_empty() {
                template.push_str(" []\n");
            } else {
                template.push('\n');
                for sound_sample in soundsamples {
                    template.push_str("- Path: ");
                    template.push_str(sound_sample.as_str());
                    template.push('\n');
                }
            }
        }
        None => {
            template.push_str(" []\n");
        }
    }

    // process sound effects
    template.push_str("SoundEffects:");
    match &chart.soundbank {
        Some(soundbank) => {
            if soundbank.sound_effects.is_empty() {
                template.push_str(" []\n");
            } else {
                template.push('\n');
                for sound_effect in &soundbank.sound_effects {
                    template.push_str(&generate_soundeffect(sound_effect.time, sound_effect.sample, sound_effect.volume));
                    template.push('\n');
                }
            }
        }
        None => {
            template.push_str(" []\n");
        }
    }
    
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
    let hitobjects: Vec<(&i32, &f32, &KeySoundRow, &Row)> = chart.hitobjects.iter_zipped().collect();
    template.reserve(hitobjects.len() * key_count as usize);
    template.push_str("HitObjects:");
    if chart.timing_points.is_bpms_empty() {
        template.push_str(" []\n");
    } else {
        template.push('\n');
        for (row_idx, (time, _, keysounds, row)) in hitobjects.iter().enumerate() {
            for (i, key) in row.iter().enumerate() {
                let keysound = if keysounds.is_empty {
                    KeySound::normal(100)
                } else {
                    keysounds[i]
                };
                match key.key_type {
                    KeyType::Normal => {
                        template.push_str(&generate_hitobject(**time, None, i, keysound));
                        template.push('\n');
                    },
                    KeyType::SliderStart => {
                        let slider_end_time = if let Some(time) = key.slider_end_time() {
                            time
                        } else {
                            find_sliderend_time(row_idx, i, &hitobjects)
                        };
                        template.push_str(&generate_hitobject(**time, Some(slider_end_time), i, keysound));
                        template.push('\n');
                    },
                    _ => continue,
                }
            }
        }
    }

    Ok(template)
}