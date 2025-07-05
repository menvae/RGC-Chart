use crate::wasm_bindgen;
use crate::models::common::{
    Row,
    Key,
    KeyType,
};
use crate::models::sound::{
    KeySoundRow,
};

#[derive(Debug)]
#[repr(C, align(64))]
pub struct HitObjectView<'a> {
    pub time: &'a i32,
    pub row: &'a [Key],
    pub beat: &'a f32,
    pub keysound: &'a KeySoundRow,
    _pad: [u8; 20],
}

impl<'a> HitObjectView<'a> {
    pub fn new(
        time: &'a i32,
        beat: &'a f32,
        row: &'a [Key],
        keysound: &'a KeySoundRow,
    ) -> Self {
        Self {
            time,
            beat,
            row,
            keysound,
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
    pub times: Vec<i32>,
    #[wasm_bindgen(skip)]
    pub rows: Vec<Row>,
    #[wasm_bindgen(skip)]
    pub beats: Vec<f32>,
    #[wasm_bindgen(skip)]
    pub keysounds: Vec<KeySoundRow>,
}

impl HitObjects {

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            times: Vec::with_capacity(capacity),
            keysounds: Vec::with_capacity(capacity),
            rows: Vec::with_capacity(capacity),
            beats: Vec::with_capacity(2_u32.pow(7) as usize)
        }
    }

    pub fn new(times: Vec<i32>, keysounds: Vec<KeySoundRow>, rows: Vec<Row>, beats: Vec<f32>) -> Self {
        Self {
            times,
            keysounds,
            rows,
            beats,
        }
    }

    #[inline]
    pub fn add_hitobject(&mut self, time: i32, beat: f32, hitsound: KeySoundRow, row: Row) {
        if row.iter().all(|&key| key.key_type == KeyType::Empty) { return; }
        self.times.push(time);
        self.keysounds.push(hitsound);
        self.beats.push(beat);
        self.rows.push(row);
    }


    /// time, beat, keysounds, row
    pub fn iter_zipped(&self) -> impl Iterator<Item = (&i32, &f32, &KeySoundRow, &Row)> {
        self.times
            .iter()
            .zip(self.beats.iter())
            .zip(self.keysounds.iter())
            .zip(self.rows.iter())
            .map(|(((time, beat), keysound), row)| (time, beat, keysound, row))
    }

    pub fn iter_views(&self) -> impl Iterator<Item = HitObjectView> {
        self.times
            .iter()
            .zip(self.keysounds.iter())
            .zip(self.rows.iter())
            .zip(self.beats.iter())
            .map(|(((time, hitsound), row), beat)| {
                HitObjectView::new(time, beat, row, hitsound)
            })
    }
}