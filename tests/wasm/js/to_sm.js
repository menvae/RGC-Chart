export async function osu_to_sm(raw_chart) {
  let chart = await wasm.parse_osu(raw_chart)
  let converted = await wasm.convert_to_sm(chart)
}
