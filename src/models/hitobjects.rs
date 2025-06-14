use crate::wasm_bindgen;
use crate::models::common::{
    Row,
    KeyType,
};

#[derive(Debug)]
#[repr(C, align(64))]
pub struct HitObjectView<'a> {
    pub time: &'a f32,
    pub row: &'a [KeyType],
    pub beat: &'a f32,
    pub hitsound: &'a [u8],
    _pad: [u8; 20],
}

impl<'a> HitObjectView<'a> {
    pub fn new(
        time: &'a f32,
        beat: &'a f32,
        key: &'a [KeyType],
        hitsound: &'a [u8],
    ) -> Self {
        Self {
            time,
            beat,
            row: key,
            hitsound,
            _pad: [0; 20],
        }
    }
}

// TODO: add wasm bindings for HitObject
// TODO: add row and object count
#[wasm_bindgen]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct HitObjects {
    #[wasm_bindgen(skip)]
    pub times: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub rows: Vec<Row>,
    #[wasm_bindgen(skip)]
    pub beats: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub hitsounds: Vec<Vec<u8>>,
}

impl HitObjects {

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            times: Vec::with_capacity(capacity),
            hitsounds: Vec::with_capacity(capacity),
            rows: Vec::with_capacity(capacity),
            beats: Vec::with_capacity(2_u32.pow(7) as usize)
        }
    }

    pub fn new(times: Vec<f32>, hitsounds: Vec<Vec<u8>>, rows: Vec<Row>, beats: Vec<f32>) -> Self {
        Self {
            times,
            hitsounds,
            rows,
            beats,
        }
    }

    #[inline]
    pub fn add_hitobject(&mut self, time: f32, beat: f32, hitsound: Vec<u8>, row: Row) {
        if row.iter().all(|&note| note == KeyType::Empty) { return; }
        self.times.push(time);
        self.hitsounds.push(hitsound);
        self.beats.push(beat);
        self.rows.push(row);
    }


    /// time, beat, hitsounds, row
    pub fn iter_zipped(&self) -> impl Iterator<Item = (&f32, &f32, &Vec<u8>, &Row)> {
        self.times
            .iter()
            .zip(self.beats.iter())
            .zip(self.hitsounds.iter())
            .zip(self.rows.iter())
            .map(|(((time, beat), hitsound), key)| (time, beat, hitsound, key))
    }

    pub fn iter_views(&self) -> impl Iterator<Item = HitObjectView> {
        self.times
            .iter()
            .zip(self.hitsounds.iter())
            .zip(self.rows.iter())
            .zip(self.beats.iter())
            .map(|(((time, hitsound), key), beat)| {
                HitObjectView::new(time, beat, key, hitsound)
            })
    }
}