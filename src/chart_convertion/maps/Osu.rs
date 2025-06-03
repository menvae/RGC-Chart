use crate::helpers::{add_key_value_template,
    parse_key_value,
    remove_comments,
    StrDefaultExtension,
    StrNumericDefaultExtension
};
use crate::chart_classes::{
    self,
    ChartDefaults,
    GameMode,
    KeyType,
    Row,
    TimingChangeType
};
use crate::rhythm::calculate_beat_from_time;
use crate::errors;
use std::fmt::{self, Display, Formatter};


#[derive(Debug)]
#[repr(C)]
struct SliderEnd {
    time: i32,
    column: usize
}

enum TimeLine {
    HitObject(HitObject),
    SliderEnd(SliderEnd),
}

impl TimeLine {
    const fn time(&self) -> i32 {
        match self {
            TimeLine::HitObject(h) => h.2,
            TimeLine::SliderEnd(s) => s.time,
        }
    }
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

#[inline(always)]
fn beatlength_to_bpm(beatlength: &f32) -> f32 {
    1.0 / beatlength * 60000.0
}

#[inline(always)]
fn beatlength_to_multiplier(beatlength: &f32) -> f32 {
    (1.0 / beatlength) * -100.0
}

#[inline(always)]
fn bpm_to_beatlength(bpm: &f32) -> f32 {
    60000.0 / bpm
}

#[inline(always)]
fn multiplier_to_beatlength(multiplier: &f32) -> f32 {
    if *multiplier == 0.0 { return -10000.0 }
    -100.0 / multiplier
}

#[inline(always)]
fn column_to_coords(column: usize, key_count: usize) -> u16 {
    (column as f32 * 512.0 / key_count as f32) as u16
}

#[inline(always)]
fn coords_to_column(coords: u32, key_count: u8) -> usize {
    (coords as f32 * key_count as f32 / 512.0) as usize
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

type TimingPoint = (i32, f32, f32, f32, i32, f32, bool, u32);
type HitObject = (u32, u32, i32, u8, u8, i32, String);
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
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

pub fn parse_event(line: &str) -> Result<Event, Box<dyn std::error::Error>> {
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

#[inline]
pub fn process_key_value<F>(
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

#[allow(clippy::single_match)]
pub(crate) fn parse_osu(raw_chart: &str) -> Result<chart_classes::Chart, Box<dyn std::error::Error>> {
    use self::OsuSection::*;
    use self::chart_classes::{Metadata, ChartInfo, TimingPoints, HitObjects};

    let uncommented_chart = remove_comments(raw_chart);
    if uncommented_chart.trim().is_empty() {
        return Err(Box::new(errors::ParseError::<GameMode>::EmptyChartData));
    }

    let mut metadata = Metadata::empty();
    let mut chartinfo = ChartInfo::empty();
    let mut timing_points = TimingPoints::with_capacity(64);
    let mut hitobjects = HitObjects::with_capacity(2048);

    let mut prev_multiplier = 0.0;
    let mut prev_is_kiai = false;

    let mut key_count = 0;

    let mut temp_row: Row = Vec::new();
    let mut temp_hitsounds: Vec<u8> = Vec::new();

    

    process_sections(&uncommented_chart, |section, content| {
        match section {
            General => {
                process_key_value(content, |key, value| {
                    match key {
                        "AudioFilename" => chartinfo.song_path = value.or_default_empty(ChartDefaults::SONG_PATH),
                        "AudioLeadIn" => {}, // todo: recheck later
                        "PreviewTime" => chartinfo.preview_time = value.or_default_empty_as(*ChartDefaults::PREVIEW_TIME),
                        "Mode" => { validate_mode_mania(value)?; },
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
                let mut prev_bpm = 120.0;
                for line in content.lines().map(str::trim) {
                    #[allow(unused)]
                    let (time, beat_length, meter, sample_set, sample_index, volume, uninherited, effects) = parse_timing_point(line)?;
                    let kiai = effects == 1 || (prev_is_kiai && effects != 0);
                    prev_is_kiai = kiai;
                    if uninherited {
                        let bpm = beatlength_to_bpm(&beat_length);
                        prev_bpm = bpm;
                        timing_points.add(time as f32, prev_multiplier, bpm, 0.0, kiai, TimingChangeType::Bpm);
                    } else {
                        let multiplier = beatlength_to_multiplier(&beat_length);
                        prev_multiplier = multiplier;
                        timing_points.add(time as f32, multiplier, prev_bpm, 0.0, kiai, TimingChangeType::Sv);
                    }
                }
                debug_assert_ne!(*timing_points.bpms.first().unwrap(), 0.0);

                let start_time = timing_points.times.first().copied().unwrap_or(0.0);
                chartinfo.audio_offset = start_time;
                
                let bpm_changes = 
                    timing_points.bpm_changes_zipped().collect::<Vec<_>>();

                let bpm_times: Vec<f32> = bpm_changes.iter().map(|(t, _, _, _, _, _)| **t).collect();
                let bpms: Vec<f32> = bpm_changes.iter().map(|(_, _, b, _, _, _)| **b).collect();
                
                timing_points.beats.iter_mut().enumerate().for_each(|(i, beat)| {
                    let time = timing_points.times[i];
                    *beat = calculate_beat_from_time(time, start_time, (&bpm_times, &bpms));
                });
            },
            
            HitObjects => {

                let timeline: Vec<TimeLine> = {

                    let lines: Vec<&str> = content.lines().map(str::trim).filter(|s| !s.is_empty()).collect();
                    let mut hitobjects = Vec::with_capacity(lines.len() * 2);
                
                    for line in lines {
                        let hit_object = parse_hitobject(line)?;

                        if hit_object.5 != 0 {
                            let slider_end = SliderEnd {
                                time: hit_object.5,
                                column: coords_to_column(hit_object.0, key_count),
                            };
                            hitobjects.push(TimeLine::SliderEnd(slider_end));
                        }

                        hitobjects.push(TimeLine::HitObject(hit_object));
                        
                    }

                    hitobjects.sort_unstable_by_key(|tl| tl.time());
                    hitobjects
                };

                let mut current_time = timeline.first().unwrap().time(); // init time, is actually previous time
                let mut time = 0;
                #[allow(unused)]
                let mut row_beat = 0.0;
                let start_time = chartinfo.audio_offset;

                for hitobject_or_sliderend in timeline {

                    time = hitobject_or_sliderend.time();        

                    if time != current_time {
                        row_beat = calculate_beat_from_time(current_time as f32, start_time, (&timing_points.times, &timing_points.bpms));
                        
                        hitobjects.add_hitobject(
                            current_time as f32,
                            row_beat,
                            temp_hitsounds.clone(),
                            temp_row.clone(),
                        );

                        current_time = time;
                        temp_row.fill(KeyType::Empty);
                        temp_hitsounds.fill(0);
                    }

                    match hitobject_or_sliderend {
                        TimeLine::HitObject(hit_object) => {
                            let column = coords_to_column(hit_object.0, key_count);
                            
                            match hit_object.3 {
                                128 => {
                                    temp_row[column] = KeyType::SliderStart;
                                },
                                n if (n & 1u8) == 1 => {
                                    if temp_row[column] != KeyType::SliderStart {
                                        temp_row[column] = KeyType::Normal;
                                    }
                                },
                                _ => {}
                            }
                            
                        },

                        TimeLine::SliderEnd(slider_end) => {
                            if temp_row[slider_end.column] != KeyType::SliderStart {
                                temp_row[slider_end.column] = KeyType::SliderEnd;
                            }
                        }
                    }
                }

                row_beat = calculate_beat_from_time(time as f32, start_time, (&timing_points.times, &timing_points.bpms));
                if current_time != 0 {
                    hitobjects.add_hitobject(
                        time as f32,
                        row_beat,
                        temp_hitsounds.clone(),
                        temp_row.clone(),
                    );
                }
            
            },
    
            _ => {},
            
        }
        Ok(())
    })?;
    
    Ok(chart_classes::Chart::new(metadata, chartinfo, timing_points, hitobjects))
}


pub(crate) fn convert_to_osu(chart: &chart_classes::Chart) -> Result<String, Box<dyn std::error::Error>> {
    let mut template = String::from("osu file format v14\n");
    let key_count = chart.chartinfo.key_count;

    // General
    template.push_str("\n[General]\n");
    add_key_value_template(&mut template,
        "AudioFilename", ": ", &chart.chartinfo.song_path, "\n");
    add_key_value_template(&mut template,
        "AudioLeadIn", ": ", "0", "\n");
    add_key_value_template(&mut template,
        "PreviewTime", ": ", &chart.chartinfo.preview_time.to_string(), "\n");
    template.push_str("Countdown: 0
SampleSet: Soft
StackLeniency: 0.7
Mode: 3
LetterboxInBreaks: 0
SpecialStyle: 0
WidescreenStoryboard: 1");
    template.push('\n');

    // Editor
    template.push_str("\n[Editor]
DistanceSpacing: 1
BeatDivisor: 4
GridSize: 4
TimelineZoom: 1
");

    // Metadata
    template.push_str("\n[Metadata]\n");
    add_key_value_template(&mut template,
        "Title", ": ", &chart.metadata.title.replace("\n", ""), "\n");
    add_key_value_template(&mut template,
        "TitleUnicode", ": ", &chart.metadata.alt_title, "\n");
    add_key_value_template(&mut template,
        "Artist", ": ", &chart.metadata.artist, "\n");
    add_key_value_template(&mut template,
        "ArtistUnicode", ": ", &chart.metadata.alt_artist, "\n");
    add_key_value_template(&mut template,
        "Creator", ": ", &chart.metadata.creator, "\n");
    add_key_value_template(&mut template,
        "Version", ": ", &chart.chartinfo.difficulty_name, "\n");
    add_key_value_template(&mut template,
        "Source", ": ", &chart.metadata.source, "\n");
    add_key_value_template(&mut template,
        "Tags", ": ", &chart.metadata.tags.join(" "), "\n");
    add_key_value_template(&mut template,
        "BeatmapID", ": ", "0", "\n");
    add_key_value_template(&mut template,
        "BeatmapSetID", ": ", "-1", "\n");

    // Difficulty
    template.push_str("\n[Difficulty]\n");
    add_key_value_template(&mut template,
        "HPDrainRate", ": ", "8.5", "\n");
    add_key_value_template(&mut template,
        "CircleSize", ": ", &key_count.to_string(), "\n");
    template.push_str("OverallDifficulty:8
ApproachRate:5
SliderMultiplier:1.4
SliderTickRate:1");
    template.push('\n');

    // Events
    template.push_str("\n[Events]\n");
    template.push_str("//Background and Video events\n");
    template.push_str(&format!("0,0,\"{}\",0,0\n", &chart.chartinfo.bg_path));
    template.push_str("//Break Periods
//Storyboard Layer 0 (Background)
//Storyboard Layer 1 (Fail)
//Storyboard Layer 2 (Pass)
//Storyboard Layer 3 (Foreground)
//Storyboard Layer 4 (Overlay)
//Storyboard Sound Samples\n");

    // process timing points
    template.push_str("\n[TimingPoints]\n");
    for timing_point in chart.timing_points.iter_views() {
        match timing_point.change_type {
            TimingChangeType::Bpm => {
                template.push_str(&format!("{},{},4,1,0,100,1,0\n",
                    timing_point.time,
                    bpm_to_beatlength(timing_point.bpm),
                ));
            },
            TimingChangeType::Sv => {
                template.push_str(&format!("{},{},4,1,0,100,0,{}\n",
                    timing_point.time,
                    multiplier_to_beatlength(timing_point.multiplier),
                    if *timing_point.kiai { 1 } else { 0 }
                ));
            },
            _ => {}
        }
    }

    template.push_str("\n[HitObjects]\n");
    let hitobjects: Vec<(&f32, &f32, &Vec<u8>, &Vec<KeyType>)> = chart.hitobjects.iter_zipped().collect();
    template.reserve(hitobjects.len() * key_count as usize);
    #[allow(unused)]
    for (row_idx, (time, beat, hitsounds, row)) in hitobjects.iter().enumerate() {
        for (i, key_type) in row.iter().enumerate() {
            let coords = column_to_coords(i, chart.chartinfo.key_count as usize);
            match key_type {
                KeyType::Normal => {
                    template.push_str(&format!("{},192,{},1,0,0:0:0:0:\n", coords, **time as i32));
                },
                KeyType::SliderStart => {
                    let slider_end_time = find_sliderend_time(row_idx, i, &hitobjects);
                    template.push_str(&format!("{},192,{},128,0,{}:0:0:0:0:\n", coords, **time as i32, slider_end_time as i32));
                },
                _ => continue,
            }
        }
    }

    Ok(template)
}

#[inline(always)]
pub fn find_sliderend_time(
    start_idx: usize,
    key_idx: usize,
    hitobjects: &[(&f32, &f32, &Vec<u8>, &Row)],
) -> f32 {
    if start_idx >= hitobjects.len() {
        return 0.0;
    }

    let start_time = hitobjects[start_idx].0;
    let slice = &hitobjects[start_idx + 1..];
    
    for (time, _, _, row) in slice {
        if row[key_idx] == KeyType::SliderEnd {
            return **time
        }
    }
    
    *start_time
}