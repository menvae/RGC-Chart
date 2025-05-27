pub mod errors;
pub mod helpers;
pub(crate) use wasm_bindgen::prelude::*;

#[macro_use]
pub mod macros;

pub mod chart_convertion {
    pub mod chart_classes;
    pub mod rhythm;
    pub(crate) mod maps;

    pub use crate::errors;
    pub use crate::helpers;
    use crate::{wasm_bindgen, JsError};

    // native parse
    #[cfg(not(target_arch = "wasm32"))]
    pub mod parse {
        use super::maps::*;

        #[inline]
        pub fn parse_osu(raw_chart: &str) -> Result<chart_classes::Chart, Box<dyn std::error::Error>> {
            Osu::parse_osu(raw_chart)
        }

        #[inline]
        pub fn parse_sm(raw_chart: &str) -> Result<chart_classes::Chart, Box<dyn std::error::Error>> {
            Stepmania::parse_sm(raw_chart)
        }
    }

    // native convert
    #[cfg(not(target_arch = "wasm32"))]
    pub mod convert {
        use super::maps::*;

        #[inline]
        pub fn convert_to_osu(chart: &chart_classes::Chart) -> Result<String, Box<dyn std::error::Error>> {
            Osu::convert_to_osu(chart)
        }

        #[inline]
        pub fn convert_to_sm(chart: &chart_classes::Chart) -> Result<String, Box<dyn std::error::Error>> {
            Stepmania::convert_to_sm(chart)
        }
    }

    // wasm32 parse
    #[cfg(target_arch = "wasm32")]
    pub mod parse {
        use super::maps::*;

        #[wasm_bindgen]
        pub fn parse_osu(raw_chart: &str) -> Result<chart_classes::Chart, JsError> {
            match Osu::parse_osu(raw_chart) {
                Ok(chart) => Ok(chart),
                Err(e) => Err(JsError::new(&e.to_string()))
            }
        }

        #[wasm_bindgen]
        pub fn parse_sm(raw_chart: &str) -> Result<chart_classes::Chart, JsError> {
            match Stepmania::parse_sm(raw_chart) {
                Ok(chart) => Ok(chart),
                Err(e) => Err(JsError::new(&e.to_string()))
            }
        }
    }

    // wasm32 convert
    #[cfg(target_arch = "wasm32")]
    pub mod convert {
        use super::maps::*;

        #[wasm_bindgen]
        pub fn convert_to_osu(chart: &chart_classes::Chart) -> Result<String, JsError> {
            match Osu::convert_to_osu(chart) {
                Ok(chart) => Ok(chart),
                Err(e) => Err(JsError::new(&e.to_string()))
            }
        }

        #[wasm_bindgen]
        pub fn convert_to_sm(chart: &chart_classes::Chart) -> Result<String, JsError> {
            match Stepmania::convert_to_sm(chart) {
                Ok(chart) => Ok(chart),
                Err(e) => Err(JsError::new(&e.to_string()))
            }
        }
    }
}

pub use chart_convertion::chart_classes;
pub use chart_convertion::rhythm;