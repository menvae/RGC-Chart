use crate::models;
use crate::models::common::{
    Row,
    KeyType,
    TimingChangeType,
};
use crate::utils::helpers::add_key_value_template;
#[allow(unused)]
use crate::errors;

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
fn find_sliderend_time(
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