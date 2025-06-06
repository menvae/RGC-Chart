use crate::wasm_bindgen;
use crate::models::common::ChartDefaults;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Metadata {
    #[wasm_bindgen(getter_with_clone)]
    pub title: String,
    #[wasm_bindgen(getter_with_clone)]
    pub alt_title: String,
    #[wasm_bindgen(getter_with_clone)]
    pub artist: String,
    #[wasm_bindgen(getter_with_clone)]
    pub alt_artist: String,
    #[wasm_bindgen(getter_with_clone)]
    pub creator: String,
    #[wasm_bindgen(getter_with_clone)]
    pub genre: String,
    #[wasm_bindgen(getter_with_clone)]
    pub tags: Vec<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub source: String,
}

impl Metadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: String,
        alt_title: String,
        artist: String,
        alt_artist: String,
        creator: String,
        genre: String,
        source: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            title,
            alt_title,
            artist,
            alt_artist,
            creator,
            genre,
            source,
            tags,  
        }
    }

    pub fn empty() -> Self {
        Self {
            title: ChartDefaults::TITLE.to_string(),
            alt_title: ChartDefaults::ALT_TITLE.to_string(),
            artist: ChartDefaults::ARTIST.to_string(),
            alt_artist: ChartDefaults::ALT_ARTIST.to_string(),
            creator: ChartDefaults::CREATOR.to_string(),
            genre: ChartDefaults::GENRE.to_string(),
            source: ChartDefaults::SOURCE.to_string(),
            tags: Vec::with_capacity(5),
        }
    }
}