//! Spectra-backed self-telemetry for [Boson]: typed event/metric schemas, Photon topic
//! helpers, and an [`OpsLog`](boson_telemetry::OpsLog) adapter that forwards Boson's own
//! runtime signals (enqueue, run, fail, lease reclaim) into [Spectra].
//!
//! [Boson]'s [`OpsLog`](boson_telemetry::OpsLog) trait is deliberately backend-agnostic:
//! Boson calls `record_counter` / `record_gauge` / `log_event` on whatever implementation the
//! host installs. This crate is that implementation for hosts that already emit their own
//! telemetry through [Spectra]: [`SpectraOpsLog`] forwards each call into `spectra-core`'s
//! non-recursive gate, and [`install_ops_log_from_env`] wires it up (or opts out) based on
//! `BOSON_TELEMETRY`.
//!
//! [Boson]: https://github.com/unified-field-dev/boson
//! [Spectra]: https://github.com/unified-field-dev/spectra
//!
//! ## Features
//!
//! - **`OpsLog` install** — [`SpectraOpsLog`] implements [`boson_telemetry::OpsLog`] by routing
//!   counters, gauges, and events through `spectra-core`.
//! - **Env-driven install** — [`install_ops_log_from_env`] reads `BOSON_TELEMETRY`
//!   (`off` / `console` / default-to-Spectra) and installs the matching log process-wide.
//! - **Typed schemas** — Spectra DSL schemas for Boson's own task/runtime tables and counters,
//!   registered via `inventory` when linked into a host.
//! - **Topic + codegen helpers** — generated `*Payload` / `*_TOPIC` DTOs and `*Recorder` /
//!   `*Logger` types, importable straight from the crate root (e.g.
//!   [`BosonTasksEnqueuedRecorder`]).
//! - **Consumer-side forwarding** — [`sink_forward`] re-dispatches raw metric/event emits onto
//!   the matching typed Spectra recorder, for sink consumers that re-emit Boson's signals
//!   downstream.
//!

//!
//! ## Concern → API
//!
//! | Concern | API |
//! |---|---|
//! | Install | [`install_ops_log_from_env`] / [`SpectraOpsLog`] |
//! | Sink forwarding | [`sink_forward`] |
//!
//! Labels for Boson's counters/gauges are supplied by callers via the `OpsLog::record_counter`
//! / `record_gauge` label slices; this crate has no dedicated label types. Lifecycle emission
//! lives in Boson (`boson-runtime`); hosts only call the install helpers.
//!
//! ## Generated schemas & topics
//!
//! Typed `*Recorder` / `*Logger` / `*Payload` / `*_TOPIC` symbols are re-exported at the crate
//! root and grouped under [`helpers`] and [`topics`]. One mid-level pattern for both surfaces:
//!
//! ```rust,no_run
//! use boson_spectra_telemetry::{
//!     BosonTasksEnqueuedPayload, BosonTasksEnqueuedRecorder, BOSON_TASKS_ENQUEUED_TOPIC,
//! };
//!
//! BosonTasksEnqueuedRecorder::record(
//!     1,
//!     serde_json::json!({"task_name": "send_email", "mode": "local"}),
//! );
//! assert_eq!(BosonTasksEnqueuedPayload::topic(), BOSON_TASKS_ENQUEUED_TOPIC);
//! ```
//!
//! See [`helpers`] for the full recorder/logger set and [`topics`] for transport DTOs.
//!
//! ## Environment
//!
//! | Variable | Values | Default |
//! |----------|--------|---------|
//! | `BOSON_TELEMETRY` | `off`, `console`, `spectra` | `spectra` (when Spectra is configured) |
//!
//! # Getting started
//!
//! Most hosts only need to install the ops log once at startup, before building the Boson
//! runtime:
//!
//! ```rust,no_run
//! // Reads `BOSON_TELEMETRY` (off / console / default-to-Spectra) and installs the
//! // matching `OpsLog` process-wide.
//! boson_spectra_telemetry::install_ops_log_from_env();
//!
//! // ... build and run your Boson host; Boson's own task/runtime events now flow
//! // through Spectra automatically.
//! ```
//!
//! Runnable: `cargo run -p boson-spectra-telemetry --example ops_log_smoke`.
//!
//! ## Where to look next
//!
//! - [`install_ops_log_from_env`] / [`SpectraOpsLog`] — process-wide `OpsLog` bootstrap
//! - [`sink_forward`] — forwarders for sink consumers that re-emit onto the typed Spectra recorders
//! - [`helpers`] / [`topics`] — generated recorders, loggers, payloads, and topic constants

#![allow(clippy::too_long_first_doc_paragraph)]

/// Typed recorders/loggers from Boson Spectra schemas.
pub mod helpers;
mod install;
mod ops_log;
mod sanitize;
// macro-generated Spectra schema types; documented via each schema's `description`
#[allow(missing_docs)]
mod schemas;
/// Forwarders for sink consumers that re-dispatch raw metric/event emits onto the matching
/// typed Spectra recorder generated from this crate's schemas.
///
/// # Examples
///
/// ```rust,no_run
/// use boson_spectra_telemetry::sink_forward;
/// use chrono::Utc;
/// use serde_json::json;
///
/// let ts = Utc::now();
/// sink_forward::forward_counter(
///     "boson_tasks_enqueued",
///     json!({"task_name": "send_email", "mode": "local"}),
///     1,
///     ts,
/// );
/// ```
pub mod sink_forward;
/// Transport `*Payload` / `*_TOPIC` DTOs from Boson Spectra schemas.
pub mod topics;

pub use helpers::*;
pub use topics::*;

pub use install::{install_ops_log_from_env, SpectraOpsLog};
