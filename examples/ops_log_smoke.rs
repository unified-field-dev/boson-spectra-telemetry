//! Install Boson `OpsLog` from env and emit one typed counter.
//!
//! ```bash
//! BOSON_TELEMETRY=console CARGO_BUILD_JOBS=1 \
//!   cargo run -p boson-spectra-telemetry --example ops_log_smoke
//! ```
//!
//! Success: `ops_log_smoke: OK`.

#![allow(clippy::print_stdout)]

use boson_spectra_telemetry::{install_ops_log_from_env, BosonTasksEnqueuedRecorder};

fn main() {
    std::env::set_var("BOSON_TELEMETRY", "console");
    install_ops_log_from_env();
    BosonTasksEnqueuedRecorder::record(
        1,
        serde_json::json!({"task_name": "example", "mode": "local"}),
    );
    println!("ops_log_smoke: OK");
}
