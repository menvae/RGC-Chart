use crate::models::{hitobjects::HitObjects, timing_points::TimingPoints, timing_points::TimingChange};
use crate::models::common::{KeyType, TimingChangeType};
use crate::utils::rhythm::calculate_beat_from_time;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimelineHitObject {
    pub time: i32,
    pub column: usize,
    pub key_type: KeyType,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimelineTimingPoint {
    pub time: i32,
    pub value: f32,
    pub change_type: TimingChangeType,
}

pub struct Timeline<Item> {
    timeline: Vec<Item>,
    is_sorted: bool,
}

pub trait TimelineItem {
    fn time(&self) -> i32;
}

impl TimelineItem for TimelineHitObject {
    fn time(&self) -> i32 {
        self.time
    }
}

impl TimelineItem for TimelineTimingPoint {
    fn time(&self) -> i32 {
        self.time
    }
}

pub type HitObjectTimeline = Timeline<TimelineHitObject>;
pub type TimingPointTimeline = Timeline<TimelineTimingPoint>;

impl<Item> Timeline<Item>
where
    Item: TimelineItem,
{
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            timeline: Vec::with_capacity(capacity),
            is_sorted: true,
        }
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            timeline: Vec::new(),
            is_sorted: true,
        }
    }

    #[inline]
    pub fn add(&mut self, timeline_object: Item) {
        if self.is_sorted && !self.timeline.is_empty() {
            self.is_sorted = timeline_object.time() >= self.timeline.last().unwrap().time();
        }
        self.timeline.push(timeline_object);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.timeline.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.timeline.is_empty()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.timeline.reserve(additional);
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.timeline.shrink_to_fit();
    }
}

impl<Item> Timeline<Item>
where
    Item: TimelineItem,
{
    #[inline]
    pub fn add_sorted(&mut self, timeline_object: Item) {
        let len = self.timeline.len();
        
        if len == 0 || timeline_object.time() >= self.timeline[len - 1].time() {
            self.timeline.push(timeline_object);
            return;
        }

        let pos = self.timeline.binary_search_by(|obj| 
            obj.time().cmp(&timeline_object.time())
        ).unwrap_or_else(|pos| pos);
        
        self.timeline.insert(pos, timeline_object);
    }

    #[inline]
    pub fn sort(&mut self) {
        if !self.is_sorted {
            self.timeline.sort_unstable_by(|a, b| a.time().cmp(&b.time()));
            self.is_sorted = true;
        }
    }
}

impl HitObjectTimeline {
    pub fn to_hitobjects(&mut self, hitobjects: &mut HitObjects,
        offset: i32, key_count: usize,
        bpms_times: &[i32], bpms: &[f32]) {
        
        if self.timeline.is_empty() {
            return;
        }

        let mut temp_row = vec![KeyType::Empty; key_count];
        let mut temp_hitsounds = vec![0; key_count];
        
        let mut current_time = self.timeline[0].time;
        let mut i = 0;
        
        while i < self.timeline.len() {            
            while i < self.timeline.len() && self.timeline[i].time == current_time {
                let obj = &self.timeline[i];
                let column = obj.column;
                
                if column < key_count {
                    match obj.key_type {
                        KeyType::Normal => {
                            if temp_row[column] != KeyType::SliderStart {
                                temp_row[column] = KeyType::Normal;
                            }
                        },
                        KeyType::SliderStart => {
                            temp_row[column] = KeyType::SliderStart;
                        },
                        KeyType::SliderEnd => {
                            if temp_row[column] != KeyType::SliderStart {
                                temp_row[column] = KeyType::SliderEnd;
                            }
                        },
                        _ => {}
                    }
                }
                i += 1;
            }
            
            let row_beat = calculate_beat_from_time(current_time, offset, (bpms_times, bpms));
            hitobjects.add_hitobject(
                current_time,
                row_beat,
                temp_hitsounds.clone(),
                temp_row.clone(),
            );
            
            if i < self.timeline.len() {
                current_time = self.timeline[i].time;
                unsafe {
                    std::ptr::write_bytes(temp_row.as_mut_ptr(), 0, temp_row.len());
                    std::ptr::write_bytes(temp_hitsounds.as_mut_ptr(), 0, temp_hitsounds.len());
                }
            }
        }
    }
}

impl TimingPointTimeline {
    pub fn to_timing_points(&mut self, timing_points: &mut TimingPoints, offset: i32) {
        if self.timeline.is_empty() {
            return;
        }

        self.sort();
        
        let mut bpm_times = Vec::new();
        let mut bpms = Vec::new();
        
        for timing_point in &self.timeline {
            match timing_point.change_type {
                TimingChangeType::Bpm => {
                    bpm_times.push(timing_point.time);
                    bpms.push(timing_point.value);
                }
                _ => {}
            }
        }
        
        let len = self.timeline.len();
        timing_points.times.reserve(len);
        timing_points.beats.reserve(len);
        timing_points.changes.reserve(len);
        
        for timing_point in &self.timeline {
            let time = timing_point.time;
            let beat = calculate_beat_from_time(time, offset, (&bpm_times, &bpms));
            
            timing_points.times.push(time);
            timing_points.beats.push(beat);
            timing_points.changes.push(TimingChange {
                value: timing_point.value,
                change_type: timing_point.change_type,
            });
        }
    }
}

impl<Item> IntoIterator for Timeline<Item> {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.into_iter()
    }
}

impl<'a, Item> IntoIterator for &'a Timeline<Item> {
    type Item = &'a Item;
    type IntoIter = std::slice::Iter<'a, Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.iter()
    }
}

impl<'a, Item> IntoIterator for &'a mut Timeline<Item> {
    type Item = &'a mut Item;
    type IntoIter = std::slice::IterMut<'a, Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.iter_mut()
    }
}

impl<Item> Index<usize> for Timeline<Item> {
    type Output = Item;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.timeline[index]
    }
}

impl<Item> IndexMut<usize> for Timeline<Item> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.timeline[index]
    }
}

impl<Item> std::ops::Deref for Timeline<Item> {
    type Target = [Item];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.timeline
    }
}

impl<Item> std::ops::DerefMut for Timeline<Item> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timeline
    }
}