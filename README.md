# RGC Chart
A library for parsing and writing charts for various rhythm games. It supports cross-platform usage including Web and Node.js environments via WebAssembly (WASM).

## Rust Usage

### Installation
Add this to your `Cargo.toml`:
```toml
[dependencies]
rgc-chart = "0.0.1"
```

Or run:
```sh
cargo add rgc-chart
```

### API Reference

#### Parsing Charts
```rust
use rgc_chart::parse;
use rgc_chart::Chart;

// Parse an osu! chart from string
let osu_chart = parse::from_osu(raw_osu_string).expect("Failed to parse osu! chart");

// Parse a Stepmania chart from string
let sm_chart = parse::from_sm(raw_sm_string).expect("Failed to parse Stepmania chart");
```

#### Writing Charts
```rust
use rgc_chart::parse;
use rgc_chart::write;
use rgc_chart::Chart;

let chart: Chart = parse::from_osu(raw_osu_string).expect("Failed to parse osu! chart");

// Write to to osu! format
let osu_string = write::to_osu(&chart);

// Write to Stepmania format
let sm_string = write::to_sm(&chart);
```

#### Chart Structure
The `Chart` struct contains all the relevant chart information:
```rust
pub struct Chart {
    pub metadata: Metadata,
    pub chartinfo: ChartInfo,
    pub timing_points: TimingPoints,
    pub hitobjects: HitObjects
}
```
The `Metadata` struct contains all the metadata related information about a specific chart, a lot of all of these can be empty:
```rust
pub struct Metadata {
    pub title: String,
    pub alt_title: String,
    pub artist: String,
    pub alt_artist: String,
    pub creator: String,
    pub genre: String,
    pub tags: Vec<String>,
    pub source: String,
}
```
The `ChartInfo` struct contains all the gameplay information about a specific chart:
```rust
pub struct ChartInfo {
    pub difficulty_name: String,
    pub bg_path: String,
    pub song_path: String,
    pub audio_offset: f32,
    pub preview_time: f32,
    pub key_count: u8,
    pub row_count: u32,
    pub object_count: u32,
}
```
The `TimingPoints` struct contains all the timing information such as bpm changes and sv:
```rust
pub struct TimingPoints {
    pub times: Vec<f32>,
    pub bpms: Vec<f32>,
    pub beats: Vec<f32>,
    pub multipliers: Vec<f32>,
    pub kiais: Vec<bool>,
    pub change_types: Vec<TimingChangeType>,
}
```
The `HitObjects` struct contains all the hitobject information.
hitobject information is stored in rows:
```rust
pub struct HitObjects {
    pub times: Vec<f32>,
    pub rows: Vec<Row>,
    pub beats: Vec<f32>,
    pub hitsounds: Vec<Vec<u8>>,
}
````

## JavaScript/TypeScript Usage

### Installation
For Node.js:
```sh
npm install rgc-chart-node
```

For web projects:
```html
<script src="https://unpkg.com/rgc-chart-web@latest/rgc_chart.js"></script>
```
or
```javascript
npm install rgc-chart-web
```
then use as an ES module

### API Reference

#### Initialization
```javascript
// For ES modules
import * as rgcChart from 'rgc-chart'; // or if not on node use the path to rgc_chart.js

// or alternatively
const rgcChart = await import('path/to/rgc_chart.js')

// For CommonJS
const rgcChart = require('rgc-chart');
```

you may need to do ``await rgcChart.default()`` after importing if you're using it in a script tag (with type="module") or you get an error like ``Uncaught TypeError: Cannot read properties of undefined (reading '__wbindgen_malloc')``

#### Parsing Charts
```javascript
// Parse an osu! chart from string
const chart = rgcChart.parse_from_osu(rawOsuString);

// Parse a Stepmania chart from string
const chart = rgcChart.parse_from_sm(rawSmString);
```

#### Writing Charts
```javascript
// write to osu! format
const osuString = rgcChart.write_to_osu(chart);

// write to Stepmania format
const smString = rgcChart.write_to_sm(chart);
```

#### TypeScript Types
The core chart library is written in Rust, but *most* types in the WASM bindings are generated for TypeScript.

[See Chart Structure](#chart-structure).
## Building

### Rust Library
```sh
cargo build
```

### WASM Bindings
1. Install wasm-pack:
```sh
cargo install wasm-pack
```
> [!IMPORTANT]  
> It's really recommended to have [wasm-opt](https://github.com/WebAssembly/binaryen) installed and added to path for the wasm build.

2. Build the package:
```sh
npm run build # debug build
npm run build-release # release build
```

3. This will build it for both node and web and the output will be in `dist-web` and `dist-node` directory.

## License
RGC uses the MIT License for all its sibiling projects.
See [LICENSE](https://github.com/menvae/RGC-Chart/blob/master/LICENSE) for more information
