use crate::telemetry::{parse_metric_key, TelemetryError, TelemetryExporter};
use std::sync::Mutex;

/// Minimal OpenTelemetry exporter placeholder. In a production system this would
/// wire to an OpenTelemetry SDK or collector exporter.
#[derive(Debug)]
pub struct OpenTelemetryExporter {
    records: Mutex<Vec<String>>,
}

impl OpenTelemetryExporter {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }

    pub fn exported_records(&self) -> Vec<String> {
        self.records.lock().unwrap().clone()
    }
}

impl TelemetryExporter for OpenTelemetryExporter {
    fn export_counter(
        &self,
        key: &str,
        value: u64,
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError> {
        let (name, parsed_labels) = parse_metric_key(key);
        let mut record = format!("counter {}={}", name, value);
        if !parsed_labels.is_empty() {
            let label_str = parsed_labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            record = format!("counter {} {{{}}}={}", name, label_str, value);
        }
        if let Some(labels) = labels {
            let label_str = labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            if !label_str.is_empty() {
                record = format!("counter {} {{{}}}={}", name, label_str, value);
            }
        }
        self.records.lock().unwrap().push(record);
        Ok(())
    }

    fn export_histogram(
        &self,
        key: &str,
        values: &[u64],
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError> {
        let (name, parsed_labels) = parse_metric_key(key);
        let label_str = parsed_labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",");
        let mut record = if label_str.is_empty() {
            format!("histogram {}={:?}", name, values)
        } else {
            format!("histogram {} {{{}}}={:?}", name, label_str, values)
        };
        if let Some(labels) = labels {
            let label_str = labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            if !label_str.is_empty() {
                record = format!("histogram {} {{{}}}={:?}", name, label_str, values);
            }
        }
        self.records.lock().unwrap().push(record);
        Ok(())
    }

    fn export_gauge(
        &self,
        key: &str,
        value: f64,
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError> {
        let (name, parsed_labels) = parse_metric_key(key);
        let mut record = format!("gauge {}={}", name, value);
        if !parsed_labels.is_empty() {
            let label_str = parsed_labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            record = format!("gauge {} {{{}}}={}", name, label_str, value);
        }
        if let Some(labels) = labels {
            let label_str = labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            if !label_str.is_empty() {
                record = format!("gauge {} {{{}}}={}", name, label_str, value);
            }
        }
        self.records.lock().unwrap().push(record);
        Ok(())
    }
}
