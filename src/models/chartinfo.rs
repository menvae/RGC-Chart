use crate::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ChartInfo {
    #[wasm_bindgen(getter_with_clone)]
    pub difficulty_name: String,
    #[wasm_bindgen(getter_with_clone)]
    pub bg_path: String,
    #[wasm_bindgen(getter_with_clone)]
    pub song_path: String,
    #[wasm_bindgen(getter_with_clone)]
    pub audio_offset: f32,
    #[wasm_bindgen(getter_with_clone)]
    pub preview_time: f32,
    #[wasm_bindgen(getter_with_clone)]
    pub key_count: u8,
    #[wasm_bindgen(getter_with_clone)]
    pub row_count: u32,
    #[wasm_bindgen(getter_with_clone)]
    pub object_count: u32,
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
impl ChartInfo {
    pub fn new(
        difficulty_name: String,
        bg_path: String,
        song_path: String,
        audio_offset: f32,
        preview_time: f32,
        key_count: u8,
        row_count: u32,
        object_count: u32,
    ) -> Self {
        Self {
            difficulty_name,
            bg_path,
            song_path,
            audio_offset,
            preview_time,
            key_count,
            row_count,
            object_count,
        }
    }

    pub fn empty() -> Self {
        Self {
            difficulty_name: String::with_capacity(20),
            bg_path: String::with_capacity(20),
            song_path: String::with_capacity(10),
            audio_offset: 0.0,
            preview_time: 0.0,
            key_count: 4,
            row_count: 0,
            object_count: 0,
        }
    }
}