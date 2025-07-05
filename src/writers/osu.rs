use crate::models;
use crate::models::common::{
    Row, TimingChangeType, KeyType
};
use crate::models::sound::SoundBank;
use models::sound::{KeySoundRow, KeySound, HitSoundType};
use crate::utils::string::add_key_value_template;
use crate::utils::time::find_sliderend_time;
#[allow(unused)]
use crate::errors;

#[inline(always)]
fn bpm_to_beatlength(bpm: &f32) -> f32 {
    60000.0 / bpm
}

#[inline(always)]
fn multiplier_to_beatlength(multiplier: &f32) -> f32 {
    if *multiplier == 0.0 { return -10000.0 }
    -100.0 / multiplier.abs()
}

#[inline(always)]
fn column_to_coords(column: usize, key_count: usize) -> u16 {
    (column as f32 * 512.0 / key_count as f32) as u16 + 64
}

fn generate_normal(coords: i32, time: i32, hitsound_str: &str, volume: u8, custom_sample: &str) -> String {
    format!("{},192,{},1,{},0:0:0:{}:{}\n", coords, time, hitsound_str, volume, custom_sample)
}

fn generate_slider(coords: i32, time: i32, hitsound_str: &str, slider_end_time: i32, volume: u8, custom_sample: &str) -> String {
    format!("{},192,{},128,{},{}:0:0:0:{}:{}\n", coords, time, hitsound_str, slider_end_time, volume, custom_sample)
}

fn generate_sb_sample(time: i32, sample_path: &str, volume: u8) -> String {
    format!("Sample,{},0,\"{}\",{}", time, sample_path, volume)
}

pub(crate) fn to_osu(chart: &models::chart::Chart) -> Result<String, Box<dyn std::error::Error>> {
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

    match &chart.soundbank {
        Some(soundbank) => {
            if soundbank.sound_effects.is_empty() { } else {
                template.push('\n');
                for sound_effect in &soundbank.sound_effects {
                    let sample_path = soundbank.get_sound_sample(sound_effect.sample).unwrap_or("".to_string());
                    template.push_str(&generate_sb_sample(sound_effect.time, &sample_path, sound_effect.volume));
                    template.push('\n');
                }
            }
        }
        None => { }
    }

    // process timing points
    template.push_str("\n[TimingPoints]\n");
    for timing_point in chart.timing_points.iter_views() {
        match timing_point.change_type {
            TimingChangeType::Bpm => {
                template.push_str(&format!("{},{},4,1,0,100,1,0\n",
                    timing_point.time,
                    bpm_to_beatlength(timing_point.value),
                ));
            },
            TimingChangeType::Sv => {
                template.push_str(&format!("{},{},4,1,0,100,0,0\n",
                    timing_point.time,
                    multiplier_to_beatlength(timing_point.value),
                ));
            },
            _ => {}
        }
    }

    template.push_str("\n[HitObjects]\n");
    let soundbank = chart.soundbank.clone().unwrap_or(SoundBank::new());
    let hitobjects: Vec<(&i32, &f32, &KeySoundRow, &Row)> = chart.hitobjects.iter_zipped().collect();
    template.reserve(hitobjects.len() * key_count as usize);
    #[allow(unused)]
    for (row_idx, (time, beat, keysounds, row)) in hitobjects.iter().enumerate() {
        for (i, key) in row.iter().enumerate() {
            let coords = column_to_coords(i, chart.chartinfo.key_count as usize);

            let keysound = if keysounds.is_empty {
                KeySound::normal(100)
            } else {
                keysounds[i]
            };
            let hitsound = keysound.hitsound_type;
            let hitsound_str = match hitsound {
                HitSoundType::Normal => "0",
                HitSoundType::Clap => "1",
                HitSoundType::Whistle => "2",
                HitSoundType::Finish => "3",
            };
            let custom_sample = if keysound.has_custom {
                soundbank.get_sound_sample(keysound.sample
                    .unwrap_or(0))
                    .unwrap_or("".to_string())
            } else {
                "".to_string()
            };
            let volume = if keysound.volume >= 100 {
                0
            } else {
                keysound.volume
            };
            
            match key.key_type {
                KeyType::Normal => {
                    template.push_str(&generate_normal(coords.into(), **time, &hitsound_str, volume, &custom_sample));
                },
                KeyType::SliderStart => {
                    let slider_end_time = if let Some(time) = key.slider_end_time() {
                        time
                    } else {
                        find_sliderend_time(row_idx, i, &hitobjects)
                    };
                    template.push_str(&generate_slider(coords.into(), **time, &hitsound_str, slider_end_time, volume, &custom_sample));
                },
                _ => continue,
            }
        }
    }

    Ok(template)
}