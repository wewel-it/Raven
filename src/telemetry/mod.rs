//! Telemetry exporter abstraction layer for runtime metrics.
//!
//! This module provides a clean dependency-injection boundary for
//! Prometheus and OpenTelemetry exporter implementations without coupling
//! the runtime to any specific backend.

use crate::agent::runtime::metrics::RuntimeMetricsSnapshot;

pub mod opentelemetry;
pub mod prometheus;

pub use opentelemetry::OpenTelemetryExporter;
pub use prometheus::PrometheusExporter;

/// Error type for telemetry exporter failures.
#[derive(Debug, thiserror::Error)]
#[error("Telemetry export failed: {0}")]
pub struct TelemetryError(pub String);

/// Trait representing a backend-agnostic exporter for runtime metrics.
pub trait TelemetryExporter: Send + Sync {
    fn export_counter(
        &self,
        key: &str,
        value: u64,
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError>;

    fn export_histogram(
        &self,
        key: &str,
        values: &[u64],
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError>;

    fn export_gauge(
        &self,
        key: &str,
        value: f64,
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError>;

    fn export_snapshot(&self, snapshot: &RuntimeMetricsSnapshot) -> Result<(), TelemetryError> {
        for (key, value) in &snapshot.counters {
            self.export_counter(key, *value, None)?;
        }

        for (key, values) in &snapshot.histograms {
            self.export_histogram(key, values, None)?;
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), TelemetryError> {
        Ok(())
    }
}

/// Parse an encoded metric key into a base name and optional labels.
///
/// Metric keys may include labels in the form `metric|key=value|other=val`.
pub fn parse_metric_key(key: &str) -> (String, Vec<(&str, &str)>) {
    let mut parts = key.split('|');
    let name = parts
        .next()
        .map(|s| s.to_string())
        .unwrap_or_else(|| key.to_string());
    let mut labels = Vec::new();
    for part in parts {
        if let Some((k, v)) = part.split_once('=') {
            labels.push((k, v));
        }
    }
    (name, labels)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExporter {
        records: std::sync::Mutex<Vec<String>>,
    }

    impl TestExporter {
        fn new() -> Self {
            Self {
                records: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn records(&self) -> Vec<String> {
            self.records.lock().unwrap().clone()
        }
    }

    impl TelemetryExporter for TestExporter {
        fn export_counter(
            &self,
            key: &str,
            value: u64,
            labels: Option<&[(&str, &str)]>,
        ) -> Result<(), TelemetryError> {
            let labels = labels
                .map(|labels| {
                    labels
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default();
            let record = if labels.is_empty() {
                format!("counter {}={}", key, value)
            } else {
                format!("counter {} {{{}}}={}", key, labels, value)
            };
            self.records.lock().unwrap().push(record);
            Ok(())
        }

        fn export_histogram(
            &self,
            key: &str,
            values: &[u64],
            labels: Option<&[(&str, &str)]>,
        ) -> Result<(), TelemetryError> {
            let labels = labels
                .map(|labels| {
                    labels
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default();
            let record = if labels.is_empty() {
                format!("histogram {}={:?}", key, values)
            } else {
                format!("histogram {} {{{}}}={:?}", key, labels, values)
            };
            self.records.lock().unwrap().push(record);
            Ok(())
        }

        fn export_gauge(
            &self,
            key: &str,
            value: f64,
            labels: Option<&[(&str, &str)]>,
        ) -> Result<(), TelemetryError> {
            let labels = labels
                .map(|labels| {
                    labels
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default();
            let record = if labels.is_empty() {
                format!("gauge {}={}", key, value)
            } else {
                format!("gauge {} {{{}}}={}", key, labels, value)
            };
            self.records.lock().unwrap().push(record);
            Ok(())
        }
    }

    #[test]
    fn parse_metric_key_with_labels() {
        let (name, labels) = parse_metric_key("workflow_started|workflow_id=workflow-1");
        assert_eq!(name, "workflow_started");
        assert_eq!(labels, vec![("workflow_id", "workflow-1")]);
    }

    #[test]
    fn exporter_can_export_snapshot() {
        let exporter = TestExporter::new();
        let mut snapshot = RuntimeMetricsSnapshot::new();
        snapshot.counters.insert("workflow_started".to_string(), 2);
        snapshot
            .histograms
            .insert("scheduler_latency_hist".to_string(), vec![1, 2, 3]);

        exporter.export_snapshot(&snapshot).unwrap();
        let records = exporter.records();
        assert!(records
            .iter()
            .any(|r| r.contains("counter workflow_started=2")));
        assert!(records
            .iter()
            .any(|r| r.contains("histogram scheduler_latency_hist=[1, 2, 3]")));
    }
}
