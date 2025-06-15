use crate::models::{hitobjects::HitObjects, timing_points::TimingPoints, timing_points::TimingChange};
use crate::models::common::{KeyType, TimingChangeType};
use crate::utils::rhythm::calculate_beat_from_time;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimelineHitObject<T> {
    pub time: T,
    pub column: usize,
    pub key_type: KeyType,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimelineTimingPoint<T> {
    pub time: T,
    pub value: f32,
    pub change_type: TimingChangeType,
}

pub struct Timeline<Item, T> {
    timeline: Vec<Item>,
    is_sorted: bool,
    _phantom: std::marker::PhantomData<T>,
}

pub trait GenericTime: Copy + PartialOrd {
    fn as_f32(self) -> f32;
    fn eq_eps(self, other: Self) -> bool;
    fn cmp_sort(a: &Self, b: &Self) -> std::cmp::Ordering;
    fn cmp_search(obj_time: &Self, target_time: &Self) -> std::cmp::Ordering;
}

pub trait TimelineItem<T> {
    fn time(&self) -> T;
}

impl<T> TimelineItem<T> for TimelineHitObject<T>
where
    T: Copy,
{
    fn time(&self) -> T {
        self.time
    }
}

impl<T> TimelineItem<T> for TimelineTimingPoint<T>
where
    T: Copy,
{
    fn time(&self) -> T {
        self.time
    }
}

impl GenericTime for i32 {
    #[inline]
    fn as_f32(self) -> f32 {
        self as f32
    }

    #[inline]
    fn eq_eps(self, other: Self) -> bool {
        self == other
    }

    #[inline]
    fn cmp_sort(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.cmp(b)
    }

    #[inline]
    fn cmp_search(obj_time: &Self, target_time: &Self) -> std::cmp::Ordering {
        obj_time.cmp(target_time)
    }
}

impl GenericTime for f32 {
    #[inline]
    fn as_f32(self) -> f32 {
        self
    }

    #[inline]
    fn eq_eps(self, other: Self) -> bool {
        (self - other).abs() < f32::EPSILON
    }

    #[inline]
    fn cmp_sort(a: &Self, b: &Self) -> std::cmp::Ordering {
        a.partial_cmp(b).unwrap()
    }

    #[inline]
    fn cmp_search(obj_time: &Self, target_time: &Self) -> std::cmp::Ordering {
        obj_time.partial_cmp(target_time).unwrap()
    }
}

pub type HitObjectTimeline<T> = Timeline<TimelineHitObject<T>, T>;
pub type TimingPointTimeline<T> = Timeline<TimelineTimingPoint<T>, T>;

impl<Item, T> Timeline<Item, T>
where
    Item: TimelineItem<T>,
    T: Copy + PartialOrd,
{
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            timeline: Vec::with_capacity(capacity),
            is_sorted: true,
            _phantom: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            timeline: Vec::new(),
            is_sorted: true,
            _phantom: std::marker::PhantomData,
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

impl<Item, T> Timeline<Item, T>
where
    Item: TimelineItem<T>,
    T: GenericTime,
{
    #[inline]
    pub fn add_sorted(&mut self, timeline_object: Item) {
        let len = self.timeline.len();
        
        if len == 0 || timeline_object.time() >= self.timeline[len - 1].time() {
            self.timeline.push(timeline_object);
            return;
        }

        let pos = self.timeline.binary_search_by(|obj| 
            T::cmp_search(&obj.time(), &timeline_object.time())
        ).unwrap_or_else(|pos| pos);
        
        self.timeline.insert(pos, timeline_object);
    }

    #[inline]
    pub fn sort(&mut self) {
        if !self.is_sorted {
            self.timeline.sort_unstable_by(|a, b| T::cmp_sort(&a.time(), &b.time()));
            self.is_sorted = true;
        }
    }
}

impl<T> HitObjectTimeline<T>
where
    T: GenericTime,
{
    pub fn to_hitobjects(&mut self, hitobjects: &mut HitObjects,
        offset: f32, key_count: usize,
        bpms_times: &[f32], bpms: &[f32]) {
        
        if self.timeline.is_empty() {
            return;
        }

        let mut temp_row = vec![KeyType::Empty; key_count];
        let mut temp_hitsounds = vec![0; key_count];
        
        let mut current_time = self.timeline[0].time;
        let mut i = 0;
        
        while i < self.timeline.len() {            
            while i < self.timeline.len() && T::eq_eps(self.timeline[i].time, current_time) {
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
            
            let row_beat = calculate_beat_from_time(current_time.as_f32(), offset, (bpms_times, bpms));
            hitobjects.add_hitobject(
                current_time.as_f32(),
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

impl<T> TimingPointTimeline<T>
where
    T: GenericTime,
{
    pub fn to_timing_points(&mut self, timing_points: &mut TimingPoints, offset: f32) {
        if self.timeline.is_empty() {
            return;
        }

        self.sort();
        
        let mut bpm_times = Vec::new();
        let mut bpms = Vec::new();
        
        for timing_point in &self.timeline {
            match timing_point.change_type {
                TimingChangeType::Bpm => {
                    bpm_times.push(timing_point.time.as_f32());
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
            let time = timing_point.time.as_f32();
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

impl<Item, T> IntoIterator for Timeline<Item, T> {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.into_iter()
    }
}

impl<'a, Item, T> IntoIterator for &'a Timeline<Item, T> {
    type Item = &'a Item;
    type IntoIter = std::slice::Iter<'a, Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.iter()
    }
}

impl<'a, Item, T> IntoIterator for &'a mut Timeline<Item, T> {
    type Item = &'a mut Item;
    type IntoIter = std::slice::IterMut<'a, Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.iter_mut()
    }
}

impl<Item, T> Index<usize> for Timeline<Item, T> {
    type Output = Item;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.timeline[index]
    }
}

impl<Item, T> IndexMut<usize> for Timeline<Item, T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.timeline[index]
    }
}

impl<Item, T> std::ops::Deref for Timeline<Item, T> {
    type Target = [Item];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.timeline
    }
}

impl<Item, T> std::ops::DerefMut for Timeline<Item, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timeline
    }
}