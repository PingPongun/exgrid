# `ExGrid` - Extraordinary Grid Layout for `egui`

[![crates.io](https://img.shields.io/crates/v/exgrid.svg)](https://crates.io/crates/exgrid)
[![Documentation](https://docs.rs/egui_struct/badge.svg)](https://docs.rs/exgrid)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/PingPongun/exgrid/blob/master/LICENSE-MIT)
[![APACHE 2.0](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/PingPongun/exgrid/blob/master/LICENSE-APACHE)

`ExGrid`- drop-in replacement for `egui::Grid` with superpowers:

- alternative/improved layout mode dedicated to narrow windows (it's not grid there, but rather some group based layout)
- supports "subdata" (rows that are indented hidden behind collapsible, but columns are still aligned with top grid)

`ExGrid` uses wrapper around `egui::Ui` named `exgrid::ExUi`, that offers some convenience functions.

## egui version

`exgrid 0.2` by default depends on `egui 0.28`. To use other versions of egui use correct feature in `Cargo.toml`, eg. to make it work with egui 0.25:

```toml
exgrid = { version = "0.2", default-features = false, features = [ "egui25" ] }
```

OR use `[patch]` section. Currently `exgrid` supports `egui 0.23-0.28`.

Default egui version feature will be updated to newest egui on semver minor release(0.3).

## License

`egui_struct` is licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
