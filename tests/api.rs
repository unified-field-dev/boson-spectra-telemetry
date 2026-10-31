//! Happy/sad coverage for `OpsLog` install, `SpectraOpsLog`, sink forward, and topics.
#![allow(missing_docs)]

use std::sync::Mutex;

use boson_spectra_telemetry::{
    install_ops_log_from_env, sink_forward, SpectraOpsLog, BOSON_HANDLER_ERROR_TOPIC,
    BOSON_REMOTE_ERRORS_TOPIC, BOSON_RUNTIME_LOG_TOPIC, BOSON_TASKS_COMPLETED_TOPIC,
    BOSON_TASKS_ENQUEUED_TOPIC, BOSON_TASKS_FAILED_TOPIC, BOSON_TASK_DURATION_MS_TOPIC,
    BOSON_TASK_LOG_TOPIC,
};
use boson_telemetry::OpsLog;
use chrono::Utc;
use serde_json::json;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn install_ops_log_off_aliases_and_spectra_default_happy() {
    let _guard = ENV_LOCK.lock().expect("env lock");

    for off in ["off", "0", "false", "none"] {
        std::env::set_var("BOSON_TELEMETRY", off);
        install_ops_log_from_env();
    }

    std::env::set_var("BOSON_TELEMETRY", "console");
    install_ops_log_from_env();

    std::env::set_var("BOSON_TELEMETRY", "spectra");
    install_ops_log_from_env();

    std::env::remove_var("BOSON_TELEMETRY");
    install_ops_log_from_env();
}

#[test]
fn spectra_ops_log_counter_gauge_event_happy() {
    let log = SpectraOpsLog::new();
    log.record_counter("boson_tasks_enqueued", &[("task_name", "t")], 1.0);
    // fractional counters truncate toward zero via the adapter
    log.record_counter("boson_tasks_enqueued", &[("task_name", "t")], 2.9);
    log.record_gauge("boson_task_duration_ms", &[("task_name", "t")], 12.0);
    log.log_event(
        "boson_runtime_log",
        &json!({"component": "coordinator", "message": "ok"}),
    );
}

#[test]
fn spectra_ops_log_unknown_names_dropped_sad() {
    let log = SpectraOpsLog::new();
    // unknown names / empty labels / empty payload are dropped — must not panic
    log.record_counter("unknown_boson_metric", &[], 0.0);
    log.record_gauge("unknown_boson_gauge", &[], -1.0);
    log.log_event("unknown_boson_event", &json!({}));
}

#[test]
fn sink_forward_known_metrics_and_events_happy() {
    let ts = Utc::now();
    let labels = json!({"task_name": "t", "mode": "local"});

    sink_forward::forward_counter("boson_tasks_enqueued", labels.clone(), 1, ts);
    sink_forward::forward_counter("boson_tasks_completed", labels.clone(), 1, ts);
    sink_forward::forward_counter("boson_tasks_failed", labels.clone(), 1, ts);
    sink_forward::forward_counter("boson_remote_errors", labels.clone(), 1, ts);
    sink_forward::forward_counter("boson_task_duration_ms", labels, 9, ts);

    sink_forward::forward_event(
        "boson_handler_error",
        &json!({"task_name": "t", "job_id": "j", "run_id": "r", "error": "e"}),
        ts,
    );
    sink_forward::forward_event(
        "boson_runtime_log",
        &json!({"component": "c", "message": "m", "mode": "local"}),
        ts,
    );
    sink_forward::forward_event(
        "boson_task_log",
        &json!({
            "task_id": "r", "job_id": "j", "task_name": "t", "attempt": 1,
            "pool": "default", "mode": "local", "duration_ms": "3",
            "status": "completed", "message": "ok"
        }),
        ts,
    );
}

#[test]
fn sink_forward_unknown_and_missing_fields_ignored_sad() {
    let ts = Utc::now();

    // unknown metric / table names are no-ops
    sink_forward::forward_counter("not_a_boson_metric", json!({}), 1, ts);
    sink_forward::forward_event("unknown_table", &json!({}), ts);

    // missing fields coerce to empty string / 0
    sink_forward::forward_event("boson_task_log", &json!({}), ts);
    sink_forward::forward_event("boson_handler_error", &json!({"task_name": null}), ts);
    sink_forward::forward_event(
        "boson_runtime_log",
        &json!({"component": [], "message": {}, "mode": true}),
        ts,
    );
}

#[test]
fn topic_constants_are_non_empty_happy() {
    for topic in [
        BOSON_TASKS_ENQUEUED_TOPIC,
        BOSON_TASKS_COMPLETED_TOPIC,
        BOSON_TASKS_FAILED_TOPIC,
        BOSON_TASK_DURATION_MS_TOPIC,
        BOSON_REMOTE_ERRORS_TOPIC,
    ] {
        assert_ne!(topic, "");
        assert!(
            topic.starts_with("spectra.metric."),
            "unexpected metric topic: {topic}"
        );
        assert!(topic.contains("boson_"));
    }
    for topic in [
        BOSON_TASK_LOG_TOPIC,
        BOSON_HANDLER_ERROR_TOPIC,
        BOSON_RUNTIME_LOG_TOPIC,
    ] {
        assert_ne!(topic, "");
        assert!(
            topic.starts_with("spectra.event."),
            "unexpected event topic: {topic}"
        );
        assert!(topic.contains("boson_"));
    }
}
