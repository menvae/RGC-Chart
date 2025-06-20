use crate::wasm_bindgen;
use crate::models::{
    metadata::Metadata,
    chartinfo::ChartInfo,
    timing_points::TimingPoints,
    hitobjects::HitObjects
};

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
    pub hitobjects: HitObjects
}

impl Chart {
    pub(crate) fn new(metadata: Metadata, chartinfo: ChartInfo, timing_points: TimingPoints, hitobjects: HitObjects) -> Self {
        Self {
            metadata,
            chartinfo,
            timing_points,
            hitobjects,
        }
    }
}
