use crate::models::{hitobjects::HitObjects};
use crate::models::common::KeyType;
use crate::utils::rhythm::calculate_beat_from_time;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TimelineHitObject<T> {
    pub time: T,
    pub column: usize,
    pub key_type: KeyType,
}

pub struct Timeline<T> {
    timeline: Vec<TimelineHitObject<T>>,
    is_sorted: bool,
}

pub trait GenericTime: Copy + PartialOrd {
    fn as_f32(self) -> f32;
    fn eq_eps(self, other: Self) -> bool;
    fn cmp_sort(a: &Self, b: &Self) -> std::cmp::Ordering;
    fn cmp_search(obj_time: &Self, target_time: &Self) -> std::cmp::Ordering;
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

impl<T: Copy + PartialOrd> Timeline<T> {
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
    pub fn add(&mut self, timeline_object: TimelineHitObject<T>) {
        if self.is_sorted && !self.timeline.is_empty() {
            self.is_sorted = timeline_object.time >= self.timeline.last().unwrap().time;
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

impl<T: GenericTime> Timeline<T> {
    #[inline]
    pub fn add_sorted(&mut self, timeline_object: TimelineHitObject<T>) {
        let len = self.timeline.len();
        
        if len == 0 || timeline_object.time >= self.timeline[len - 1].time {
            self.timeline.push(timeline_object);
            return;
        }

        let pos = self.timeline.binary_search_by(|obj| 
            T::cmp_search(&obj.time, &timeline_object.time)
        ).unwrap_or_else(|pos| pos);
        
        self.timeline.insert(pos, timeline_object);
    }

    #[inline]
    pub fn sort(&mut self) {
        if !self.is_sorted {
            self.timeline.sort_unstable_by(|a, b| T::cmp_sort(&a.time, &b.time));
            self.is_sorted = true;
        }
    }

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
                // Reset arrays efficiently
                unsafe {
                    std::ptr::write_bytes(temp_row.as_mut_ptr(), 0, temp_row.len());
                    std::ptr::write_bytes(temp_hitsounds.as_mut_ptr(), 0, temp_hitsounds.len());
                }
            }
        }
    }
}

impl<T> IntoIterator for Timeline<T> {
    type Item = TimelineHitObject<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Timeline<T> {
    type Item = &'a TimelineHitObject<T>;
    type IntoIter = std::slice::Iter<'a, TimelineHitObject<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Timeline<T> {
    type Item = &'a mut TimelineHitObject<T>;
    type IntoIter = std::slice::IterMut<'a, TimelineHitObject<T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.timeline.iter_mut()
    }
}

impl<T> Index<usize> for Timeline<T> {
    type Output = TimelineHitObject<T>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.timeline[index]
    }
}

impl<T> IndexMut<usize> for Timeline<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.timeline[index]
    }
}

impl<T> std::ops::Deref for Timeline<T> {
    type Target = [TimelineHitObject<T>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.timeline
    }
}

impl<T> std::ops::DerefMut for Timeline<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timeline
    }
}