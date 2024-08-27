# `ExGrid` - Extraordinary Grid Layout for `egui`

[![crates.io](https://img.shields.io/crates/v/exgrid.svg)](https://crates.io/crates/exgrid)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/PingPongun/exgrid/blob/master/LICENSE)

ExGrid- drop-in replacement for egui::Grid with superpowers:

- alternative/improved layout mode dedicated to narrow windows (it's not grid there, but rather some group based layout)
- supports "subdata" (rows that are indented hidden behind collapsible, but columns are still aligned with top grid)

## egui version

`exgrid 0.2` by default depends on `egui 0.28`. To use other versions of egui use correct feature in `Cargo.toml`, eg. to make it work with egui 0.25:

```toml
exgrid = { version = "0.2", default-features = false, features = [ "egui25" ] }
```

OR use `[patch]` section. Currently `exgrid` supports `egui 0.23-0.28`.

Default egui version feature will be updated to newest egui on semver minor release(0.3).
