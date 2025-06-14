export async function osu_to_sm(raw_chart) {
  let chart = await wasm.parse_from_osu(raw_chart)
  let converted = await wasm.write_to_sm(chart)
}

export async function qua_to_sm(raw_chart) {
  let chart = await wasm.parse_from_qua(raw_chart)
  let converted = await wasm.write_to_sm(chart)
}
