pub mod errors;
pub(crate) use wasm_bindgen::prelude::*;

#[macro_use]
pub mod macros;

pub type Chart = crate::models::chart::Chart;

pub(crate) mod parsers;
pub(crate) mod writers;
pub mod models;
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
pub mod parse {
    use crate::parsers;

    #[inline]
    pub fn from_osu(raw_chart: &str) -> Result<crate::Chart, Box<dyn std::error::Error>> {
        parsers::osu::from_osu(raw_chart)
    }

    #[inline]
    pub fn from_sm(raw_chart: &str) -> Result<crate::Chart, Box<dyn std::error::Error>> {
        parsers::stepmania::from_sm(raw_chart)
    }

    #[inline]
    pub fn from_qua(raw_chart: &str) -> Result<crate::Chart, Box<dyn std::error::Error>> {
        parsers::quaver::from_qua(raw_chart)
    }
}

#[cfg(target_arch = "wasm32")]
pub mod parse {
    use crate::wasm_bindgen;
    use crate::JsError;
    use crate::parsers;

    #[wasm_bindgen]
    pub fn parse_from_osu(raw_chart: &str) -> Result<crate::Chart, JsError> {
        match parsers::osu::from_osu(raw_chart) {
            Ok(chart) => Ok(chart),
            Err(e) => Err(JsError::new(&e.to_string()))
        }
    }

    #[wasm_bindgen]
    pub fn parse_from_sm(raw_chart: &str) -> Result<crate::Chart, JsError> {
        match parsers::stepmania::from_sm(raw_chart) {
            Ok(chart) => Ok(chart),
            Err(e) => Err(JsError::new(&e.to_string()))
        }
    }

    #[wasm_bindgen]
    pub fn parse_from_qua(raw_chart: &str) -> Result<crate::Chart, JsError> {
        match parsers::quaver::from_qua(raw_chart) {
            Ok(chart) => Ok(chart),
            Err(e) => Err(JsError::new(&e.to_string()))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod write {
    use crate::writers;

    #[inline]
    pub fn to_osu(chart: &crate::Chart) -> Result<String, Box<dyn std::error::Error>> {
        writers::osu::to_osu(chart)
    }

    #[inline]
    pub fn to_sm(chart: &crate::Chart) -> Result<String, Box<dyn std::error::Error>> {
        writers::stepmania::to_sm(chart)
    }

    #[inline]
    pub fn to_qua(chart: &crate::Chart) -> Result<String, Box<dyn std::error::Error>> {
        writers::quaver::to_qua(chart)
    }
}

#[cfg(target_arch = "wasm32")]
pub mod write {
    use crate::wasm_bindgen;
    use crate::JsError;
    use crate::writers;

    #[wasm_bindgen]
    pub fn write_to_osu(chart: &crate::Chart) -> Result<String, JsError> {
        match writers::osu::to_osu(chart) {
            Ok(chart) => Ok(chart),
            Err(e) => Err(JsError::new(&e.to_string()))
        }
    }

    #[wasm_bindgen]
    pub fn write_to_sm(chart: &crate::Chart) -> Result<String, JsError> {
        match writers::stepmania::to_sm(chart) {
            Ok(chart) => Ok(chart),
            Err(e) => Err(JsError::new(&e.to_string()))
        }
    }

    #[wasm_bindgen]
    pub fn write_to_qua(chart: &crate::Chart) -> Result<String, JsError> {
        match writers::quaver::to_qua(chart) {
            Ok(chart) => Ok(chart),
            Err(e) => Err(JsError::new(&e.to_string()))
        }
    }
}
