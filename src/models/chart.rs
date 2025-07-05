use crate::models::{
    chartinfo::ChartInfo, hitobjects::HitObjects, metadata::Metadata, sound::SoundBank,
    timing_points::TimingPoints,
};
use crate::wasm_bindgen;

// TODO: maybe don't use getter_with_clone?
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Chart {
    #[wasm_bindgen(getter_with_clone)]
    pub metadata: Metadata,
    #[wasm_bindgen(getter_with_clone)]
    pub chartinfo: ChartInfo,
    #[wasm_bindgen(getter_with_clone)]
    pub timing_points: TimingPoints,
    #[wasm_bindgen(getter_with_clone)]
    pub hitobjects: HitObjects,
    #[wasm_bindgen(getter_with_clone)]
    pub soundbank: Option<SoundBank>,
}

impl Chart {
    pub(crate) fn new(
        metadata: Metadata,
        chartinfo: ChartInfo,
        timing_points: TimingPoints,
        hitobjects: HitObjects,
        soundbank: Option<SoundBank>,
    ) -> Self {
        Self {
            metadata,
            chartinfo,
            timing_points,
            hitobjects,
            soundbank,
        }
    }
}
