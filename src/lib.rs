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
//! - **Env-resolved telemetry install** — Reads `BOSON_TELEMETRY` at host boot and installs the matching
//!   process-wide `OpsLog` before the Boson runtime starts.
//!   [Get started](#env-driven-install)
//! - **Spectra `OpsLog` adapter** — [`SpectraOpsLog`] implements [`boson_telemetry::OpsLog`] when you wire
//!   the Spectra adapter yourself instead of using the env helper.
//!   [Get started](#direct-ops-log)
//! - **Consumer-side forwarding** — [`sink_forward`] re-dispatches raw metric and event emits
//!   onto the matching typed Spectra recorder for sink consumers that re-emit Boson signals
//!   downstream. [Get started](#sink-forwarding)
//! - **Topic + codegen helpers** — Generated `*Recorder` / `*Logger` / `*Payload` / `*_TOPIC`
//!   symbols for explicit Boson telemetry emits from host or test code.
//!   [Get started](#typed-recorders)
//! - **Typed schemas** — Spectra DSL schemas for Boson's task/runtime tables and counters,
//!   registered via `inventory` when linked into a host.
//!
//! # Getting started
//!
//! Most hosts install the ops log once at startup, then let Boson's runtime emit through Spectra
//! automatically. Pick the env helper for production hosts or wire [`SpectraOpsLog`] directly when
//! tests need a fixed backend.
//!
//! ## Env-driven install
//!
//! [`install_ops_log_from_env`] is the default host path: it resolves `BOSON_TELEMETRY` once at
//! process boot and registers the matching `OpsLog` before you build the Boson runtime, so enqueue,
//! run, and fail signals flow through Spectra for the process lifetime.
//!
//! Prerequisites: Spectra must already be booted in the host process when `BOSON_TELEMETRY` is
//! unset or set to `spectra`. Set `off` or `console` to disable or print locally.
//!
//! ```rust,no_run
//! // Call before constructing the Boson runtime.
//! boson_spectra_telemetry::install_ops_log_from_env();
//! let telemetry = std::env::var("BOSON_TELEMETRY").unwrap_or_else(|_| "spectra".into());
//! assert!(!telemetry.trim().is_empty());
//! ```
//!
//! Runnable: `cargo run -p boson-spectra-telemetry --example ops_log_smoke`.
//!
//! Next: [Direct ops log](#direct-ops-log) when you need explicit wiring in tests.
//!
//! ## Direct ops log
//!
//! [`SpectraOpsLog`] is for hosts or tests that install `OpsLog` without reading
//! `BOSON_TELEMETRY`. Construct the adapter and pass it to [`boson_telemetry::install_ops_log`]
//! before Boson starts emitting counters and events.
//!
//! Prerequisites: Spectra booted when using the default Spectra backend. Labels for counters and
//! gauges come from Boson callers via `OpsLog::record_counter` / `record_gauge` label slices.
//!
//! ```rust,no_run
//! use std::sync::Arc;
//!
//! use boson_spectra_telemetry::SpectraOpsLog;
//! use boson_telemetry::{install_ops_log, OpsLog};
//!
//! let log = SpectraOpsLog::new();
//! install_ops_log(Arc::new(log));
//! log.record_counter(
//!     "boson_tasks_enqueued",
//!     &[("task_name", "send_email"), ("mode", "local")],
//!     1.0,
//! );
//! let metric = "boson_tasks_enqueued";
//! assert_eq!(metric, "boson_tasks_enqueued");
//! ```
//!
//! Next: [Sink forwarding](#sink-forwarding) when a Spectra sink re-emits raw Boson
//! metric names.
//!
//! ## Sink forwarding
//!
//! [`sink_forward`] maps raw Boson metric and event names onto this crate's typed
//! `*Recorder` / `*Logger` helpers. Use it from Spectra sink consumers that receive generic
//! emits and need to re-emit onto the Boson schema surface downstream.
//!
//! Prerequisites: the incoming metric or table name must match a Boson schema this crate
//! registers (`boson_tasks_enqueued`, `boson_handler_error`, and the other Boson topics).
//!
//! ```rust,no_run
//! use boson_spectra_telemetry::sink_forward;
//! use chrono::Utc;
//! use serde_json::json;
//!
//! sink_forward::forward_counter(
//!     "boson_tasks_enqueued",
//!     json!({"task_name": "send_email", "mode": "local"}),
//!     1,
//!     Utc::now(),
//! );
//! let forwarded = "boson_tasks_enqueued";
//! assert_eq!(forwarded, "boson_tasks_enqueued");
//! ```
//!
//! API reference: [`sink_forward`] module. Next: [Typed recorders](#typed-recorders) when you
//! emit Boson telemetry directly without a sink hop.
//!
//! ## Typed recorders
//!
//! Generated `*Recorder` and `*Logger` types under [`helpers`] emit Boson counters and events
//! with typed labels and topic constants from [`topics`]. Call them from host code or tests when
//! you need an explicit emit instead of relying on Boson's runtime `OpsLog` path.
//!
//! Prerequisites: Spectra booted in the process. Import recorders from the crate root or
//! [`helpers`]; transport DTOs and `*_TOPIC` constants live in [`topics`].
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
//! # Feature flags
//!
//! This crate has no Cargo feature flags.

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
