use crate::models;
use crate::models::common::{
    ChartDefaults,
    GameMode,
    KeyType,
    TimingChangeType
};
use crate::utils::string::{
    remove_comments,
    parse_key_value,
    trim_split_iter,
    StrDefaultExtension,
    StrNumericDefaultExtension,
};
use crate::utils::rhythm::calculate_beat_from_time;
use crate::errors;


type TimingPoint = (f32, f32, TimingChangeType);
type HitObject = (f32, usize, Vec<u8>, f32);

fn process_sections<F>(raw_chart: &str, mut lambda: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&str, &str) -> Result<(), Box<dyn std::error::Error>>,
{
    let lines = raw_chart.lines();
    let mut current_content = String::with_capacity(raw_chart.len());
    let mut current_section: Option<&str> = None;
    
    for line in lines {
        if let Some(colon_index) = line.find(':') {
            let header = line[..colon_index].trim_end();
            let content = line[colon_index + 1..].trim();
            
            if content.is_empty() {
                if let Some(section) = current_section {
                    lambda(section, current_content.trim())?;
                    current_content.clear();
                }
                current_section = Some(header);
            } else {
                if let Some(section) = current_section {
                    if line.starts_with(char::is_whitespace) || line.starts_with('-') {
                        current_content.push_str(line);
                        current_content.push('\n');
                    } else {
                        lambda(section, current_content.trim())?;
                        current_content.clear();
                        current_section = None;
                        lambda(header, content)?;
                    }
                } else {
                    lambda(header, content)?;
                }
            }
        } else if current_section.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    
    if let Some(section) = current_section {
        lambda(section, current_content.trim())?;
    }
    Ok(())
}

fn parse_timing_point(raw: &str) -> Result<TimingPoint, Box<dyn std::error::Error>> {
    let mut time = 0f32;
    let mut timing_point_value = 1.0;
    let mut change_type = TimingChangeType::Bpm;

    for s in raw.split('\n') {
        let (key, value) = parse_key_value(s);
        match key {
            "StartTime" => {
                time = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse time in SliderVelocities or TimingPoints: {}", value)
                    )
                })?;
            },
            "Bpm" => {
                change_type = TimingChangeType::Bpm;
                timing_point_value = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse BPM: {}", value)
                    )
                })?;
            },
            "Multiplier" => {
                change_type = TimingChangeType::Sv;
                timing_point_value = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse multiplier: {}", value)
                    )
                })?;
            },
            _ => {},
        }
    }

    Ok((time, timing_point_value, change_type))
}

fn parse_hitobject(raw: &str) -> Result<HitObject, Box<dyn std::error::Error>> {
    let mut time = 0f32;
    let mut lane: usize = 1;
    let mut end_time = 0.0;

    for s in raw.split('\n') {
        let (key, value) = parse_key_value(s);
        match key {
            "StartTime" => {
                time = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse time in HitObjects: {}", value)
                    )
                })?;
            },
            "Lane" => {
                lane = value.parse::<usize>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't Lane: {}", value)
                    )
                })?;
            },
            "EndTime" => {
                end_time = value.parse::<f32>().map_err(|_| {
                    errors::ParseError::<GameMode>::InvalidChart(
                        format!("Couldn't parse end_time in HitObjects: {}", value)
                    )
                })?;
            }
            _ => {},
        }
    }

    Ok((time, lane - 1, vec![0u8], end_time))
}

fn process_timing_points(timing_points: &mut models::timing_points::TimingPoints,
    chartinfo: &mut models::chartinfo::ChartInfo,
    start_multiplier: f32, raw_bpms: &str, raw_sv: &str) -> Result<(), Box<dyn std::error::Error>> {
    
   
    let mut content = String::with_capacity(raw_bpms.len() + raw_sv.len());
    let mut prev_bpm = 1.0;
    let mut prev_multiplier = start_multiplier;
    let kiai = false;

    if raw_bpms.trim().is_empty() || raw_bpms == "[]" {
        return Err(Box::new(errors::ParseError::<GameMode>::InvalidChart("No BPM Data Provided.".to_string())));
    }

    if raw_sv.trim().is_empty() || raw_sv == "[]" {
        content.push_str(raw_bpms);
    } else {
        content.push_str(raw_bpms);
        content.push('\n');
        content.push_str(raw_sv);
    }

    let seperated_timing_points = trim_split_iter(content.split("-"), true);
    for timing_point in seperated_timing_points {
        let (time, value, change_type) = parse_timing_point(timing_point)?;
        if change_type == TimingChangeType::Bpm {
            prev_bpm = value;
            timing_points.add(time as f32, prev_multiplier, value, 0.0, kiai, change_type);
        } else {
            prev_multiplier = value;
            timing_points.add(time as f32, value, prev_bpm, 0.0, kiai, change_type);
        }
    }

    let start_time = timing_points.times.first().copied().unwrap_or(0.0);
    chartinfo.audio_offset = start_time;
    
    let bpm_changes = timing_points.bpm_changes_zipped().collect::<Vec<_>>();
        let bpm_times: Vec<f32> = bpm_changes.iter().map(|(t, _, _, _, _, _)| **t).collect();
        let bpms: Vec<f32> = bpm_changes.iter().map(|(_, _, b, _, _, _)| **b).collect();
        
        timing_points.beats.iter_mut().enumerate().for_each(|(i, beat)| {
            let time = timing_points.times[i];
            *beat = calculate_beat_from_time(time, start_time, (&bpm_times, &bpms));
        });
    Ok(())
}

fn process_notes(hitobjects: &mut models::hitobjects::HitObjects,
    chartinfo: &mut models::chartinfo::ChartInfo,
    bpms_times: &Vec<f32>,
    bpms: &Vec<f32>,
    raw_notes: &str) -> Result<(), Box<dyn std::error::Error>> {
        use models::timeline::{Timeline, TimelineHitObject};
        let mut key_count = chartinfo.key_count as usize;
        
        let mut timeline: Timeline<f32> = Timeline::with_capacity((raw_notes.len() / 3) as usize);

        let seperated_hitobjects = trim_split_iter(raw_notes.split("-"), true);
        for hitobject in seperated_hitobjects {
        let (object_time, lane, _key_sounds, slider_end_time) = parse_hitobject(hitobject)?;
            if lane > 6 {
                key_count = lane+1; // Quaver, what the actual FUCK is wrong with you..
            }
            if slider_end_time != 0.0 {
                let slider = TimelineHitObject {
                    time: object_time,
                    column: lane,
                    key_type: KeyType::SliderStart,
                };

                let slider_end = TimelineHitObject {
                    time: slider_end_time,
                    column: lane,
                    key_type: KeyType::SliderEnd,
                };
            
                timeline.add_sorted(slider);
                timeline.add_sorted(slider_end);
            } else {
                timeline.add_sorted(
                TimelineHitObject {
                        time: object_time,
                        column: lane,
                        key_type: KeyType::Normal,
                    }
                );
            }
        }

        chartinfo.key_count = key_count as u8;
        timeline.to_hitobjects(hitobjects, chartinfo.audio_offset, key_count, bpms_times, bpms);
        
        Ok(())
}

pub(crate) fn from_qua(raw_chart: &str) -> Result<models::chart::Chart, Box<dyn std::error::Error>>  {
    use models::{metadata::Metadata, chartinfo::ChartInfo, timing_points::TimingPoints, hitobjects::HitObjects, chart::Chart};

    let uncommented_chart = remove_comments(raw_chart, "#");
    if uncommented_chart.trim().is_empty() {
        return Err(Box::new(errors::ParseError::<GameMode>::EmptyChartData));
    }

    let mut metadata = Metadata::empty();
    let mut chartinfo = ChartInfo::empty();
    let mut timing_points = TimingPoints::with_capacity(64);
    let mut hitobjects = HitObjects::with_capacity(2048);
    let mut start_multiplier = 1.0;

    let mut raw_bpms = String::new();
    let mut raw_sv = String::new();
    let mut raw_notes = String::new();

    process_sections(&uncommented_chart, |header, content| {
        match header {
            "AudioFile" => chartinfo.song_path = content.or_default_empty(ChartDefaults::SONG_PATH),
            "SongPreviewTime" => chartinfo.preview_time = content.or_default_empty_as::<f32>(*ChartDefaults::PREVIEW_TIME),
            "BackgroundFile" => chartinfo.bg_path = content.or_default_empty(ChartDefaults::SONG_PATH),
            "Mode" => {chartinfo.key_count = content.or_default_empty_as::<u8>(*ChartDefaults::KEY_COUNT)},
            "Title" => metadata.title = content.or_default_empty(ChartDefaults::TITLE),
            "Artist" => metadata.artist = content.or_default_empty(ChartDefaults::ARTIST),
            "Source" => metadata.source = content.or_default_empty(ChartDefaults::SOURCE),
            "Tags" => metadata.tags = content.split(' ').map(String::from).collect(),
            "Creator" => metadata.creator = content.or_default_empty(ChartDefaults::CREATOR),
            "DifficultyName" => chartinfo.difficulty_name = content.or_default_empty(ChartDefaults::DIFFICULTY_NAME),
            "BPMDoesNotAffectScrollVelocity" => {}, // TODO: maybe do something with these later
            "InitialScrollVelocity" => start_multiplier = content.or_default_empty_as::<f32>(1.0),
            "CustomAudioSamples" => {},
            "SoundEffects" => {},
            "TimingPoints" => raw_bpms = content.or_default_empty(ChartDefaults::RAW_BPMS),
            "SliderVelocities" => raw_sv = content.or_default_empty(ChartDefaults::RAW_SV),
            "HitObjects" => raw_notes = content.to_string(), // TODO: this is really bad because we're copying the whole note data which is very bad for maps with a lot of objects,
                                                            //need to make this somehow not copy without making rust mad..
            _ => {},
        }
        Ok(())
    })?;

    process_timing_points(&mut timing_points, &mut chartinfo, start_multiplier, &raw_bpms, &raw_sv)?;
    process_notes(&mut hitobjects,&mut chartinfo, &timing_points.times, &timing_points.bpms, &raw_notes)?;

    Ok(Chart::new(metadata, chartinfo, timing_points, hitobjects))
}
