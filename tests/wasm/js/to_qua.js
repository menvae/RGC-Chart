export async function osu_to_qua(raw_chart) {
  let chart = await wasm.parse_from_osu(raw_chart)
  let converted = await wasm.write_to_qua(chart)
}

export async function sm_to_qua(raw_chart) {
  let chart = await wasm.parse_from_sm(raw_chart)
  let converted = await wasm.write_to_qua(chart)
}
