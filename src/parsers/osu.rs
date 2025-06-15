use std::fmt::{self, Display, Formatter};
use crate::models;
use crate::models::common::{
    GameMode,
    Row,
    ChartDefaults,
    KeyType,
    TimingChangeType,
};
use crate::utils::string::{
    remove_comments,
    parse_key_value,
    StrDefaultExtension,
    StrNumericDefaultExtension,
};
use crate::utils::rhythm::{
    calculate_beat_from_time,
};
use crate::errors;

#[derive(Debug, PartialEq, Eq)]
enum OsuSection {
    Unknown,
    General,
    Editor,
    Metadata,
    Difficulty,
    Events,
    TimingPoints,
    HitObjects
}

#[allow(unused)]
#[derive(Debug)]
enum OsuMode {
    Standard,
    Taiko,
    Catch,
    Mania,
    Unknown,
}

impl Display for OsuMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mode_str = match self {
            OsuMode::Standard => "Standard",
            OsuMode::Taiko => "Taiko",
            OsuMode::Catch => "Catch",
            OsuMode::Mania => "Mania",
            OsuMode::Unknown => "Unknown",
        };
        write!(f, "{mode_str}")
    }
}

type TimingPoint = (i32, f32, f32, f32, i32, f32, bool, u32);
type HitObject = (u32, u32, i32, u8, u8, i32, String);
#[derive(Debug, PartialEq, Eq)]
enum Event {
    Background {
        start_time: u32,
        filename: String,
        x_offset: i32,
        y_offset: i32,
    },
    Video {
        start_time: u32,
        filename: String,
    },
    Break {
        start_time: u32,
        end_time: u32,
    },
    Unknown(String),
}

#[inline(always)]
fn beatlength_to_bpm(beatlength: &f32) -> f32 {
    1.0 / beatlength * 60000.0
}

#[inline(always)]
fn beatlength_to_multiplier(beatlength: &f32) -> f32 {
    (1.0 / beatlength) * -100.0
}

#[inline(always)]
fn coords_to_column(coords: u32, key_count: u8) -> usize {
    (coords as f32 * key_count as f32 / 512.0) as usize
}

fn validate_mode_mania(mode: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use self::OsuMode::*;

    if mode != "3" {
        let current_mode = match mode {
            "0" => Standard,
            "1" => Taiko,
            "2" => Catch,
            _ => Unknown,
        };
        return Err( Box::new(errors::ParseError::InvalidMode(current_mode.to_string(), GameMode::Mania)) );
    }
    Ok(true)
}


#[inline]
fn process_key_value<F>(
    content: &str, 
    mut lambda: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&str, &str) -> Result<(), Box<dyn std::error::Error>>,
{
    for line in content.lines().map(str::trim) {
        let (key, value) = parse_key_value(line);
        lambda(key, value)?;
    }
    Ok(())
}

fn process_sections<F>(raw_chart: &str, mut lambda: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(OsuSection, &str) -> Result<(), Box<dyn std::error::Error>>,
{
    let mut current_content = String::with_capacity(raw_chart.len());
    let mut current_section = OsuSection::Unknown;
    
    for line in raw_chart.lines().map(str::trim) {
        // if line.is_empty() {
        //     continue;
        // }
        
        let new_section = match line {
            "[General]" => OsuSection::General,
            "[Editor]" => OsuSection::Editor,
            "[Metadata]" => OsuSection::Metadata,
            "[Difficulty]" => OsuSection::Difficulty,
            "[Events]" => OsuSection::Events,
            "[TimingPoints]" => OsuSection::TimingPoints,
            "[HitObjects]" => OsuSection::HitObjects,
            _ => OsuSection::Unknown,
        };
        
        if new_section != OsuSection::Unknown {
            if !current_content.is_empty() {
                lambda(current_section, &current_content)?;
                current_content.clear();
            }
            current_section = new_section;
        } else {
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
        }
    }
    
    if !current_content.is_empty() {
        lambda(current_section, &current_content)?;
    }
    
    Ok(())
}

macro_rules! parse_next {
    ($field:literal, $iter:expr, $raw:expr) => {{
        let value_str = $iter.next()
            .ok_or_else(|| errors::ParseError::<GameMode>::InvalidChart(
                format!("Missing {}: '{}'", $field, $raw)
            ))?;
        value_str.parse()
            .map_err(|e| errors::ParseError::<GameMode>::InvalidChart(
                format!("Failed to parse {} '{}' in '{}': {}", $field, value_str, $raw, e)
            ))?
    }};
}

fn parse_timing_point(raw: &str) -> Result<TimingPoint, Box<dyn std::error::Error>> {
    let mut components = raw.split(',');

    Ok((
        parse_next!("time", components, raw),
        parse_next!("beat length", components, raw),
        parse_next!("meter", components, raw),
        parse_next!("sample set", components, raw),
        parse_next!("sample index", components, raw),
        parse_next!("volume", components, raw),
        match components.next()
            .ok_or_else(|| errors::ParseError::<GameMode>::InvalidChart(
                format!("Missing uninherited in timing point: '{}'", raw)
            ))? {
            "0" => false,
            "1" => true,
            s => return Err(Box::new(errors::ParseError::<GameMode>::InvalidChart(
                format!("Uninherited flag should be 0 or 1 but got '{}' in '{}'", s, raw)
            ))),
        },
        parse_next!("effects", components, raw),
    ))
}

fn parse_hitobject(raw: &str) -> Result<HitObject, Box<dyn std::error::Error>> {
    let mut components = raw.splitn(2, ":").flat_map(|s| s.split(','));

    let x = parse_next!("X coordinate", components, raw);
    let y = parse_next!("Y coordinate", components, raw);
    let time = parse_next!("time", components, raw);
    let note_type: u8 = parse_next!("note type", components, raw);
    let hit_sound = parse_next!("hit sound", components, raw);

    if note_type == 128 {
        let object_params = parse_next!("object params", components, raw);
        let hit_sample = parse_next!("hit sample", components, raw);
        Ok((x, y, time, note_type, hit_sound, object_params, hit_sample))
    } else {
        let hit_sample = parse_next!("hit sample", components, raw);
        Ok((x, y, time, note_type, hit_sound, 0, hit_sample))
    }
}

fn parse_event(line: &str) -> Result<Event, Box<dyn std::error::Error>> {
    let line = line.trim();
    
    if line.is_empty() {
        return Ok(Event::Unknown("".to_string()));
    }
    
    let mut components = line.split(',');
    let first_component = components.next().unwrap_or("").trim();
    
    match first_component {
        "0" => {
            Ok(Event::Background {
                start_time: parse_next!("start_time", components, line),
                filename: components.next()
                    .ok_or("Missing filename")?
                    .trim_matches('"')
                    .to_string(),
                x_offset: parse_next!("x_offset", components, line),
                y_offset: parse_next!("y_offset", components, line),
            })
        },
        "Video" => {
            Ok(Event::Video {
                start_time: parse_next!("start_time", components, line),
                filename: components.next()
                    .ok_or("Missing filename")?
                    .trim_matches('"')
                    .to_string(),
            })
        },
        "2" => {
            Ok(Event::Break {
                start_time: parse_next!("start_time", components, line),
                end_time: parse_next!("end_time", components, line),
            })
        },
        _ => Ok(Event::Unknown(line.to_string())),
    }
}

#[allow(clippy::single_match)]
pub(crate) fn from_osu(raw_chart: &str) -> Result<models::chart::Chart, Box<dyn std::error::Error>> {
    use self::OsuSection::*;
    use models::{metadata::Metadata, chartinfo::ChartInfo, timing_points::TimingPoints, timing_points::TimingChange, hitobjects::HitObjects, chart::Chart};

    let uncommented_chart = remove_comments(raw_chart, "//");
    if uncommented_chart.trim().is_empty() {
        return Err(Box::new(errors::ParseError::<GameMode>::EmptyChartData));
    }

    let mut metadata = Metadata::empty();
    let mut chartinfo = ChartInfo::empty();
    let mut timing_points = TimingPoints::with_capacity(64);
    let mut hitobjects = HitObjects::with_capacity(2048);

    let mut key_count = 0;

    let mut temp_row: Row = Vec::new();
    let mut temp_hitsounds: Vec<u8> = Vec::new();

    process_sections(&uncommented_chart, |section, content| {
        match section {
            General => {
                process_key_value(content, |key, value| {
                    match key {
                        "AudioFilename" => chartinfo.song_path = value.or_default_empty(ChartDefaults::SONG_PATH),
                        "AudioLeadIn" => {}, // TODO: recheck later
                        "PreviewTime" => chartinfo.preview_time = value.or_default_empty_as(*ChartDefaults::PREVIEW_TIME),
                        "Mode" => { validate_mode_mania(value)?; }, // TODO: modify this when adding taiko support later
                        _ => {},
                    }
                    Ok(())
                })?;
            },
    
            Metadata => {
                process_key_value(content, |key, value| {
                    match key {
                        "Title" => metadata.title = value.or_default_empty(ChartDefaults::TITLE),
                        "TitleUnicode" => metadata.alt_title = value.or_default_empty(ChartDefaults::ALT_TITLE),
                        "Artist" => metadata.artist = value.or_default_empty(ChartDefaults::ARTIST),
                        "ArtistUnicode" => metadata.alt_artist = value.or_default_empty(ChartDefaults::ALT_ARTIST),
                        "Creator" => metadata.creator = value.or_default_empty(ChartDefaults::CREATOR),
                        "Version" => chartinfo.difficulty_name = value.or_default_empty(ChartDefaults::DIFFICULTY_NAME),
                        "Tags" => metadata.tags = value.split(' ').map(String::from).collect(),
                        _ => {},
                    }
                    Ok(())
                })?;
            },
            Events => {
                for line in content.lines() {
                    let line = line.trim();
                    
                    let event = parse_event(line)?;
                    
                    #[allow(unused)]
                    match event {
                        Event::Background { start_time, filename, x_offset, y_offset } => {
                            if chartinfo.bg_path.is_empty() {
                                chartinfo.bg_path = filename
                            }
                        },
                        Event::Video { start_time, filename } => {},
                        Event::Break { start_time, end_time } => {},
                        Event::Unknown(raw) => {}
                    }

                }
            }
    
            Difficulty => {
                process_key_value(content, |key, value| {
                    match key {
                        "CircleSize" => {
                            key_count = value.or_default_empty_as::<f32>(*ChartDefaults::KEY_COUNT as f32) as u8;
                            temp_row = vec![KeyType::Empty; key_count as usize];
                            temp_hitsounds = vec![0; key_count as usize];
                            chartinfo.key_count = key_count;
                        },
                        _ => {},
                    }
                    Ok(())
                })?;
            },
            TimingPoints => {
                for line in content.lines().map(str::trim) {
                        #[allow(unused)]
                        let (time, beat_length, meter, sample_set, sample_index, volume, uninherited, effects) = parse_timing_point(line)?;
                        
                        if uninherited {
                            let bpm = beatlength_to_bpm(&beat_length);
                            timing_points.add(time as f32, 0.0, TimingChange {
                                change_type: TimingChangeType::Bpm,
                                value: bpm,
                            });
                        } else {
                            let multiplier = beatlength_to_multiplier(&beat_length);
                            timing_points.add(time as f32, 0.0, TimingChange {
                                change_type: TimingChangeType::Sv,
                                value: multiplier,
                            });
                        }
                    }

                    let start_time = timing_points.times.first().copied().unwrap_or(0.0);
                    chartinfo.audio_offset = start_time;

                    let bpm_changes = timing_points.bpm_changes_zipped().collect::<Vec<_>>();

                    let bpm_times: Vec<f32> = bpm_changes.iter().map(|(t, _, _)| **t).collect();
                    let bpms: Vec<f32> = bpm_changes.iter().map(|(_, _, change)| change.value).collect();

                    timing_points.beats.iter_mut().enumerate().for_each(|(i, beat)| {
                        let time = timing_points.times[i];
                        *beat = calculate_beat_from_time(time, start_time, (&bpm_times, &bpms));
                });
            },
            
            HitObjects => {
                use models::timeline::{Timeline, TimelineHitObject};

                let lines: Vec<&str> = content.lines().map(str::trim).filter(|s| !s.is_empty()).collect();
                let mut timeline: Timeline<i32> = Timeline::with_capacity(lines.len());

                for line in lines {
                    let hit_object: HitObject = parse_hitobject(line)?;
                    let slider_end_time = hit_object.5;
                    let object_time = hit_object.2;
                    let object_column = coords_to_column(hit_object.0, key_count);

                    if hit_object.3 == 128 {
                        let slider = TimelineHitObject {
                            time: object_time,
                            column: object_column,
                            key_type: KeyType::SliderStart,
                        };

                        let slider_end = TimelineHitObject {
                            time: slider_end_time,
                            column: object_column,
                            key_type: KeyType::SliderEnd,
                        };
                    
                        timeline.add_sorted(slider);
                        timeline.add_sorted(slider_end);
                    } else if (hit_object.3 & 1u8) == 1 {
                        timeline.add_sorted(
                        TimelineHitObject {
                                time: object_time,
                                column: object_column,
                                key_type: KeyType::Normal,
                            }
                        );
                    }
                }

                timeline.to_hitobjects(&mut hitobjects,
                    chartinfo.audio_offset,chartinfo.key_count as usize,
                    &timing_points.times, &timing_points.bpms());
            },
            _ => {},
            
        }
        Ok(())
    })?;
    
    Ok(Chart::new(metadata, chartinfo, timing_points, hitobjects))
}
