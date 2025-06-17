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
    pub audio_offset: i32,
    #[wasm_bindgen(getter_with_clone)]
    pub preview_time: i32,
    #[wasm_bindgen(getter_with_clone)]
    pub key_count: u8,
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
impl ChartInfo {
    pub fn new(
        difficulty_name: String,
        bg_path: String,
        song_path: String,
        audio_offset: i32,
        preview_time: i32,
        key_count: u8,
    ) -> Self {
        Self {
            difficulty_name,
            bg_path,
            song_path,
            audio_offset,
            preview_time,
            key_count,
        }
    }

    pub fn empty() -> Self {
        Self {
            difficulty_name: String::with_capacity(20),
            bg_path: String::with_capacity(20),
            song_path: String::with_capacity(10),
            audio_offset: 0,
            preview_time: 0,
            key_count: 4,
        }
    }
}