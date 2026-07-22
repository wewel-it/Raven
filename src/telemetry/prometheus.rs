use crate::telemetry::{TelemetryError, TelemetryExporter};
use prometheus::{Encoder, GaugeVec, IntCounterVec, Opts, Registry, TextEncoder};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const PROMETHEUS_BUCKETS: &[f64] = &[10.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 5000.0, 10000.0];

fn extract_label_names_and_values(labels: Option<&[(&str, &str)]>) -> (Vec<String>, Vec<String>) {
    let mut items: Vec<(&str, &str)> = labels.unwrap_or(&[]).to_vec();
    items.sort_by(|(a, _), (b, _)| a.cmp(b));
    let names = items.iter().map(|(k, _)| k.to_string()).collect();
    let values = items.iter().map(|(_, v)| v.to_string()).collect();
    (names, values)
}

fn cache_key(name: &str, label_names: &[String]) -> String {
    let mut key = String::from(name);
    for label in label_names {
        key.push('|');
        key.push_str(label);
    }
    key
}

/// Prometheus exporter implementation using the `prometheus` registry.
#[derive(Debug)]
pub struct PrometheusExporter {
    registry: Registry,
    counters: Mutex<HashMap<String, Arc<IntCounterVec>>>,
    gauges: Mutex<HashMap<String, Arc<GaugeVec>>>,
}

impl PrometheusExporter {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            counters: Mutex::new(HashMap::new()),
            gauges: Mutex::new(HashMap::new()),
        }
    }

    pub fn registry(&self) -> Registry {
        self.registry.clone()
    }

    pub fn gather_text(&self) -> Result<String, TelemetryError> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| TelemetryError(format!("prometheus encode failed: {}", e)))?;
        String::from_utf8(buffer).map_err(|e| TelemetryError(format!("utf8 error: {}", e)))
    }

    fn counter_for(
        &self,
        name: &str,
        help: &str,
        label_names: &[String],
    ) -> Result<Arc<IntCounterVec>, TelemetryError> {
        let key = cache_key(name, label_names);
        let mut guard = self.counters.lock().unwrap();
        if let Some(counter) = guard.get(&key) {
            return Ok(counter.clone());
        }
        let opts = Opts::new(name, help);
        let counter_vec = IntCounterVec::new(
            opts,
            &label_names.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .map_err(|e| TelemetryError(format!("prometheus counter register failed: {}", e)))?;
        self.registry
            .register(Box::new(counter_vec.clone()))
            .map_err(|e| TelemetryError(format!("prometheus registry register failed: {}", e)))?;
        let arc = Arc::new(counter_vec);
        guard.insert(key, arc.clone());
        Ok(arc)
    }

    fn gauge_for(
        &self,
        name: &str,
        help: &str,
        label_names: &[String],
    ) -> Result<Arc<GaugeVec>, TelemetryError> {
        let key = cache_key(name, label_names);
        let mut guard = self.gauges.lock().unwrap();
        if let Some(gauge) = guard.get(&key) {
            return Ok(gauge.clone());
        }
        let opts = Opts::new(name, help);
        let gauge_vec = GaugeVec::new(
            opts,
            &label_names.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .map_err(|e| TelemetryError(format!("prometheus gauge register failed: {}", e)))?;
        self.registry
            .register(Box::new(gauge_vec.clone()))
            .map_err(|e| TelemetryError(format!("prometheus registry register failed: {}", e)))?;
        let arc = Arc::new(gauge_vec);
        guard.insert(key, arc.clone());
        Ok(arc)
    }
}

impl TelemetryExporter for PrometheusExporter {
    fn export_counter(
        &self,
        _key: &str,
        value: u64,
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError> {
        let (name, label_values) = extract_label_names_and_values(labels);
        let counter = self.counter_for(&name.join("_"), "runtime counter", &name)?;
        let metric = counter
            .get_metric_with_label_values(
                &label_values.iter().map(String::as_str).collect::<Vec<_>>(),
            )
            .map_err(|e| TelemetryError(format!("prometheus get counter metric failed: {}", e)))?;
        metric.inc_by(value);
        Ok(())
    }

    fn export_histogram(
        &self,
        key: &str,
        values: &[u64],
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError> {
        let (name, mut label_values) = extract_label_names_and_values(labels);
        let mut label_names = name.clone();
        label_names.push("le".to_string());
        let gauge = self.gauge_for(
            &format!("{}_bucket", key),
            "runtime histogram bucket",
            &label_names,
        )?;

        for (index, count) in values.iter().enumerate() {
            let le = if index < PROMETHEUS_BUCKETS.len() {
                PROMETHEUS_BUCKETS[index].to_string()
            } else {
                "+Inf".to_string()
            };
            label_values.push(le);
            let metric = gauge
                .get_metric_with_label_values(
                    &label_values.iter().map(String::as_str).collect::<Vec<_>>(),
                )
                .map_err(|e| {
                    TelemetryError(format!(
                        "prometheus get histogram bucket metric failed: {}",
                        e
                    ))
                })?;
            metric.set(*count as f64);
            label_values.pop();
        }
        Ok(())
    }

    fn export_gauge(
        &self,
        _key: &str,
        value: f64,
        labels: Option<&[(&str, &str)]>,
    ) -> Result<(), TelemetryError> {
        let (name, label_values) = extract_label_names_and_values(labels);
        let gauge = self.gauge_for(&name.join("_"), "runtime gauge", &name)?;
        let metric = gauge
            .get_metric_with_label_values(
                &label_values.iter().map(String::as_str).collect::<Vec<_>>(),
            )
            .map_err(|e| TelemetryError(format!("prometheus get gauge metric failed: {}", e)))?;
        metric.set(value);
        Ok(())
    }
}
