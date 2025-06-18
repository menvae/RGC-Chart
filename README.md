# RGC Chart
A library for parsing and writing charts for various rhythm games. It supports cross-platform usage including Web and Node.js environments via WebAssembly (WASM).

## Table of Contents

- [Rust Usage](#rust-usage)
    - [Installation](#installation)
    - [API Reference](#api-reference)
        - [Parsing Charts](#parsing-charts)
        - [Writing Charts](#writing-charts)
        - [Chart Structure](#chart-structure)
- [JavaScript/TypeScript Usage](#javascripttypescript-usage)
    - [Installation](#installation-1)
    - [API Reference](#api-reference-1)
        - [Initialization](#initialization)
        - [Parsing Charts](#parsing-charts-1)
        - [Writing Charts](#writing-charts-1)
        - [TypeScript Types](#typescript-types)
- [Building](#building)
    - [Rust Library](#rust-library)
    - [WASM Bindings](#wasm-bindings)
- [License](#license)

## Rust Usage

### Installation
Add this to your `Cargo.toml`:
```toml
[dependencies]
rgc-chart = "0.0.6"
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

// Parse a Quaver chart from string
let qua_chart = parse::from_qua(raw_qua_string).expect("Failed to parse Quaver chart");
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

// Write to Quaver format
let qua_string = write::to_qua(&chart);
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
}
```
The `TimingPoints` struct contains all the timing information such as bpm changes and sv:
```rust
pub struct TimingPoints {
    pub times: Vec<f32>,
    pub beats: Vec<f32>,
    pub changes: Vec<TimingChange>,
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
npm install rgc-chart-nodejs
```

For web projects:
```html
<script src="https://unpkg.com/rgc-chart-browser@latest/rgc_chart.js"></script>
```
or
```javascript
npm install rgc-chart-browser
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

you may need to do ``await rgcChart.default()`` after importing if you've imported it in a script tag (with type="module") or you get an error like ``Uncaught TypeError: Cannot read properties of undefined (reading '__wbindgen_malloc')``

#### Parsing Charts
```javascript
// Parse an osu! chart from string
const chart = rgcChart.parse_from_osu(rawOsuString);

// Parse a Stepmania chart from string
const chart = rgcChart.parse_from_sm(rawSmString);

// Parse a Quaver chart from string
const chart = rgcChart.parse_from_qua(rawQuaString);
```

#### Writing Charts
```javascript
// write to osu! format
const osuString = rgcChart.write_to_osu(chart);

// write to Stepmania format
const smString = rgcChart.write_to_sm(chart);

// write to Quaver format
const quaString = rgcChart.write_to_qua(chart);
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

3. This will build it for both node and browser and the output will be in `dist-web` and `dist-node` directory.

## License
RGC uses the MIT License for all its sibiling projects.
See [LICENSE](https://github.com/menvae/RGC-Chart/blob/master/LICENSE) for more information
