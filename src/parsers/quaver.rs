use crate::models;
use crate::models::common::{
    ChartDefaults,
    GameMode,
    Key,
    TimingChangeType
};
use crate::models::sound::{HitSoundType, KeySound, SoundBank, SoundEffect};
use crate::utils::string::{
    remove_comments,
    parse_key_value,
    trim_split_iter,
    StrDefaultExtension,
    StrNumericDefaultExtension,
};
use crate::errors;

type HitSample = (usize, u8);
type TimingPoint = (f32, f32, TimingChangeType);
type HitObject = (f32, usize, Option<KeySound>, f32);

fn process_sections<F>(raw_chart: &str, mut lambda: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&str, &str) -> Result<(), Box<dyn std::error::Error>>,
{
    let lines = raw_chart.lines();
    let mut current_content = String::with_capacity(raw_chart.len());
    let mut current_section: Option<&str> = None;
    
    for line in lines {
        let is_indented = line.starts_with(|c: char| c.is_whitespace());
        
        if is_indented || line.starts_with('-') {
            if current_section.is_some() {
                current_content.push_str(line);
                current_content.push('\n');
            }
        } else if let Some(colon_index) = line.find(':') {
            let header = line[..colon_index].trim_end();
            let content = line[colon_index + 1..].trim();
            
            if let Some(section) = current_section {
                lambda(section, current_content.trim())?;
                current_content.clear();
            }
            
            if content.is_empty() {
                current_section = Some(header);
            } else {
                current_section = None;
                lambda(header, content)?;
            }
        } else {
            if let Some(section) = current_section {
                lambda(section, current_content.trim())?;
                current_content.clear();
            }
            current_section = None;
            lambda(line.trim(), "")?;
        }
    }
    
    if let Some(section) = current_section {
        lambda(section, current_content.trim())?;
    }
    
    Ok(())
}

fn split_indent(raw: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    
    if raw.starts_with("- ") {
        start = 2;
    }
    
    let mut search_from = 0;
    while let Some(pos) = raw[search_from..].find("\n- ") {
        let absolute_pos = search_from + pos;
        
        if absolute_pos >= start {
            let section = raw[start..absolute_pos].trim();
            if !section.is_empty() {
                result.push(section);
            }
        }
        
        start = absolute_pos + 3;
        search_from = start;
    }
    
    if start < raw.len() {
        let section = raw[start..].trim();
        if !section.is_empty() {
            result.push(section);
        }
    }
    
    result
}

fn parse_timing_point(raw: &str) -> Result<TimingPoint, Box<dyn std::error::Error>> {
    let mut time = 0f32;
    let mut timing_point_value = 1.0;
    
    for s in raw.split('\n') {
        let (key, value) = parse_key_value(s);
        match key {
            "StartTime" => {
                time = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse time in TimingPoints: '{}'", value)
                    )
                })?;
            },
            "Bpm" => {
                timing_point_value = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse BPM: '{}'", value)
                    )
                })?;
            },
            _ => {},
        }
    }

    Ok((time, timing_point_value, TimingChangeType::Bpm))
}

fn parse_sv(raw: &str) -> Result<TimingPoint, Box<dyn std::error::Error>> {
    let mut time = 0f32;
    let mut sv_value = 1.0;
    
    for s in raw.trim().split('\n') {
        let (key, value) = parse_key_value(s);
        match key {
            "StartTime" => {
                time = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse time in SliderVelocities: '{}'", value)
                    )
                })?;
            },
            "Multiplier" => {
                sv_value = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse multiplier: '{}'", value)
                    )
                })?;
            },

            _ => {},
        }
    }

    Ok((time, sv_value, TimingChangeType::Sv))
}

fn parse_soundeffect(raw: &str) -> Result<SoundEffect, Box<dyn std::error::Error>> {
    let mut time = 0f32;
    let mut sample_index = 1;
    let mut volume = 100;
    
    for s in raw.trim().split('\n') {
        let (key, value) = parse_key_value(s);
        match key {
            "StartTime" => {
                time = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse time in Soundeffects: '{}'", value)
                    )
                })?;
            },
            "Sample" => {
                sample_index = value.parse::<usize>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse Sample: '{}'", value)
                    )
                })?;
            },
            "Volume" => {
                volume = value.parse::<u8>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse Volume: '{}'", value)
                    )
                })?;
            },

            _ => {},
        }
    }

    Ok(SoundEffect { time: time as i32, volume: volume, sample: sample_index })
}

fn parse_keysound(raw: &str) -> Result<HitSample, Box<dyn std::error::Error>> {
    let mut sample_index = 1;
    let mut volume = 100;

    let no_whitespace: String = raw.chars()
        .filter(|c| *c == '\n' || !c.is_whitespace())
        .collect();
    
    let without_dash = if no_whitespace.starts_with('-') {
        &no_whitespace[1..]
    } else {
        &no_whitespace
    };
    
    for line in without_dash.lines() {
        let (key, value) = parse_key_value(line);
        
        match key {
            "Sample" => {sample_index = value.parse::<usize>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't sample in Keysounds: '{}'", value)
                    )
                })?;},
            "Volume" => {volume = value.parse::<u8>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't volume in Keysounds: '{}'", value)
                    )
                })?.clamp(0, 100);},
            _ => {},
        }
    }
    Ok((sample_index, volume))
}

fn parse_hitobject(raw: &str) -> Result<HitObject, Box<dyn std::error::Error>> {
    let mut time = 0f32;
    let mut lane: usize = 1;
    let mut end_time = 0.0;
    let mut hitsound_type = HitSoundType::Normal;
    let mut keysound_sample: Option<HitSample> = None;

    let mut lines = raw.trim().split('\n');
    
    while let Some(line) = lines.next() {
        let (key, value) = parse_key_value(line);
        
        match key {
            "StartTime" => {
                time = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse time in HitObjects: '{}'", value)
                    )
                })?;
            },
            "Lane" => {
                lane = value.parse::<usize>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse Lane: '{}'", value)
                    )
                })?;
            },
            "EndTime" => {
                end_time = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse end_time in HitObjects: '{}'", value)
                    )
                })?;
            },
            "HitSound" => {
                hitsound_type = match value.to_lowercase().as_str() {
                    "clap" => HitSoundType::Clap,
                    "whistle" => HitSoundType::Whistle,
                    "finish" => HitSoundType::Finish,
                    _ => HitSoundType::Normal,
                };
            },
            "KeySounds" => {
                let key_sounds = lines.collect::<Vec<_>>().join("\n");
                if key_sounds.trim().is_empty() {
                    break;
                }
                keysound_sample = Some(parse_keysound(key_sounds.as_str())?);
                break;
            },
            _ => {},
        }
    }

    let keysound = if keysound_sample.is_some() {
        let keysound_unwraped = keysound_sample.unwrap();
        Some(KeySound::with_custom(keysound_unwraped.1, keysound_unwraped.0, Some(hitsound_type)))
    } else {
        if hitsound_type == HitSoundType::Normal {
            None
        } else {
            Some(KeySound::of_type(100, hitsound_type))
        }
    };

    Ok((time, lane - 1, keysound, end_time))
}

fn process_timing_points(timeline: &mut models::timeline::TimingPointTimeline,
    chartinfo: &mut models::chartinfo::ChartInfo,
    raw_bpms: &str) -> Result<(), Box<dyn std::error::Error>> {
    use models::timeline::TimelineTimingPoint;

    let trimmed_raw = raw_bpms.trim();

    if trimmed_raw == "[]" || trimmed_raw.is_empty() {
        return Err(Box::new( errors::ParseError::InvalidChart::<GameMode>("No BPM data provided in the chart".to_string()) ))
    }
    
    let seperated_timing_points = trim_split_iter(raw_bpms.split("- "), true);
    
    for timing_point in seperated_timing_points {
        let (time, value, change_type) = parse_timing_point(timing_point)?;
        timeline.add_sorted(TimelineTimingPoint {
            time: time as i32,
            value,
            change_type,
        });
    }

    let start_time = if timeline.is_empty() {
        0
    } else {
        timeline[0].time
    };

    chartinfo.audio_offset = start_time;
    Ok(())
}

fn process_sv(timeline: &mut models::timeline::TimingPointTimeline,
    raw_sv: &str) -> Result<(), Box<dyn std::error::Error>> {
    use models::timeline::TimelineTimingPoint;

    let trimmed_raw = raw_sv.trim();

    if trimmed_raw == "[]" || trimmed_raw.is_empty() {
        return Ok(());
    }
    
    let seperated_timing_points = trim_split_iter(raw_sv.split("- "), true);
    
    for timing_point in seperated_timing_points {
        let (time, value, change_type) = parse_sv(timing_point)?;
        timeline.add_sorted(TimelineTimingPoint {
            time: time as i32,
            value,
            change_type,
        });
    }
    Ok(())
}

fn process_soundeffects(raw: &str, soundbank: &mut SoundBank) -> Result<(), Box<dyn std::error::Error>> {
    let trimmed_raw = raw.trim();

    if trimmed_raw == "[]" || trimmed_raw.is_empty() {
        return Ok(());
    }

    let seperated_se = trim_split_iter(raw.split("- "), true);
    for sound_effect in seperated_se {
        let soundeffect = parse_soundeffect(sound_effect)?;
        soundbank.add_sound_effect(soundeffect);
    }
    Ok(())
}

fn process_samples(raw: &str, soundbank: &mut SoundBank) -> Result<(), Box<dyn std::error::Error>> {
    let trimmed_raw = raw.trim();

    if trimmed_raw == "[]" || trimmed_raw.is_empty() {
        return Ok(());
    }

    let seperated_samples = trim_split_iter(raw.split("- "), true);
    for sample in seperated_samples {
        let (key, sample_path) = parse_key_value(sample);
        if key.to_ascii_lowercase() == "path" {
            soundbank.add_sound_sample(sample_path.to_string());
        }
    }
    Ok(())
}

fn process_notes(hitobjects: &mut models::hitobjects::HitObjects,
    chartinfo: &mut models::chartinfo::ChartInfo,
    bpms_times: &Vec<i32>,
    bpms: &Vec<f32>,
    raw_notes: &str) -> Result<(), Box<dyn std::error::Error>> {
        use models::timeline::{HitObjectTimeline, TimelineHitObject};
        let mut key_count = chartinfo.key_count as usize;
        
        let mut timeline: HitObjectTimeline = HitObjectTimeline::with_capacity((raw_notes.len() / 3) as usize);

        let seperated_hitobjects = split_indent(raw_notes);
        for hitobject in seperated_hitobjects {
        let (object_time, lane, key_sounds, slider_end_time) = parse_hitobject(hitobject)?;
            if lane > 6 {
                key_count = lane+1; // Quaver, what the actual FUCK is wrong with you..
            }
            if slider_end_time != 0.0 {
                let slider = TimelineHitObject {
                    time: object_time as i32,
                    column: lane,
                    key: Key::slider_start(Some(slider_end_time as i32)),
                    keysound: key_sounds
                };

                let slider_end = TimelineHitObject {
                    time: slider_end_time as i32,
                    column: lane,
                    key: Key::slider_end(),
                    keysound: None
                };
            
                timeline.add_sorted(slider);
                timeline.add_sorted(slider_end);
            } else {
                timeline.add_sorted(
                TimelineHitObject {
                        time: object_time as i32,
                        column: lane,
                        key: Key::normal(),
                        keysound: key_sounds
                    }
                );
            }
        }

        chartinfo.key_count = key_count as u8;
        timeline.to_hitobjects(hitobjects, chartinfo.audio_offset, key_count, bpms_times, bpms);
        
        Ok(())
}

pub(crate) fn from_qua(raw_chart: &str) -> Result<models::chart::Chart, Box<dyn std::error::Error>>  {
    use models::{
        metadata::Metadata,
        chartinfo::ChartInfo,
        timing_points::TimingPoints,
        timeline::TimingPointTimeline,
        hitobjects::HitObjects,
        chart::Chart,
        sound::SoundBank
    };

    let uncommented_chart = remove_comments(raw_chart, "#");
    if uncommented_chart.trim().is_empty() {
        return Err(Box::new(errors::ParseError::<GameMode>::EmptyChartData));
    }

    let mut metadata = Metadata::empty();
    let mut chartinfo = ChartInfo::empty();
    let mut timing_points = TimingPoints::with_capacity(64);
    let mut hitobjects = HitObjects::with_capacity(2048);
    let mut soundbank = SoundBank::new();
    let mut timeline: TimingPointTimeline = TimingPointTimeline::with_capacity(64);

    process_sections(&uncommented_chart, |header, content| {
        match header {
            "AudioFile" => {
                chartinfo.song_path = content.or_default_empty(ChartDefaults::SONG_PATH);
                soundbank.audio_tracks.push(chartinfo.song_path.clone());
            },
            "SongPreviewTime" => chartinfo.preview_time = content.or_default_empty_as::<i32>(*ChartDefaults::PREVIEW_TIME),
            "BackgroundFile" => chartinfo.bg_path = content.or_default_empty(ChartDefaults::SONG_PATH),
            "Mode" => {
                if content == "Keys4" {
                    chartinfo.key_count = 4;
                } else if content == "Keys7" {
                    chartinfo.key_count = 7;
                } else {
                    return Err( Box::new(errors::ParseError::<GameMode>::InvalidChart("Quaver only supports Keys4 and Keys7 for Mode".to_string())) );
                }
            },
            "Title" => metadata.title = content.or_default_empty(ChartDefaults::TITLE),
            "Artist" => metadata.artist = content.or_default_empty(ChartDefaults::ARTIST),
            "Source" => metadata.source = content.or_default_empty(ChartDefaults::SOURCE),
            "Tags" => metadata.tags = content.split(' ').map(String::from).collect(),
            "Creator" => metadata.creator = content.or_default_empty(ChartDefaults::CREATOR),
            "DifficultyName" => chartinfo.difficulty_name = content.or_default_empty(ChartDefaults::DIFFICULTY_NAME),
            "BPMDoesNotAffectScrollVelocity" => {}, // TODO: maybe do something with these later
            "InitialScrollVelocity" => {},
            "CustomAudioSamples" => process_samples(content, &mut soundbank)?,
            "SoundEffects" => process_soundeffects(content, &mut soundbank)?,
            "TimingPoints" => process_timing_points(&mut timeline, &mut chartinfo, content)?,
            "SliderVelocities" => process_sv(&mut timeline, content)?,
            "HitObjects" => {
                timeline.to_timing_points(&mut timing_points, chartinfo.audio_offset);
                process_notes(&mut hitobjects,&mut chartinfo, &timing_points.times, &timing_points.bpms(), content)?;
            }
            _ => {},
        }
        Ok(())
    })?;    

    Ok(Chart::new(metadata, chartinfo, timing_points, hitobjects, Some(soundbank)))
}
