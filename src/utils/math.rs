#[inline(always)]
pub fn approx_eq(a: f32, b: f32, margin: f32) -> bool {
    (a - b).abs() <= margin
}

#[inline(always)]
pub fn thresholded_ceil(value: f32, threshold: f32) -> f32 {
    let fractional = value.fract();
    if fractional >= threshold {
        value.floor() + 1.0
    } else {
        value
    }
}