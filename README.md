# boson-spectra-telemetry

[![CI](https://github.com/unified-field-dev/boson-spectra-telemetry/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/boson-spectra-telemetry/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[GitHub](https://github.com/unified-field-dev/boson-spectra-telemetry) · `cargo doc -p boson-spectra-telemetry --open`

Spectra-backed telemetry for [Boson](https://github.com/unified-field-dev/boson): DSL schemas, typed Photon topic helpers, and OpsLog install so task enqueue/run/fail metrics land in Spectra without hand-rolled sinks.

```toml
boson-spectra-telemetry = { git = "https://github.com/unified-field-dev/boson-spectra-telemetry" }
```

```rust
use boson_spectra_telemetry::install_ops_log_from_env;

// Reads `BOSON_TELEMETRY` (off / console / Spectra).
// Install the Spectra sink first, then OpsLog install, then build Boson.
install_ops_log_from_env();
```

Hosts call the install helpers; lifecycle emission lives in Boson (`boson-runtime`).

## About

- Spectra DSL schemas under `schemas/` (inventory-registered when linked)
- `SpectraOpsLog` / `install_ops_log_from_env` for host bootstrap
- Generated `*Recorder` / `*Logger` / topic helpers
- `sink_forward` onto typed Spectra recorders

## Examples

Runnable smoke: [examples/README.md](examples/README.md).

## Verify

```bash
export CARGO_BUILD_JOBS=1
cargo test
```

## License

MIT. See [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
