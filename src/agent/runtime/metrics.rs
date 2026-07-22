use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Lightweight snapshot of metrics for reporting and assertions.
#[derive(Debug, Clone)]
pub struct RuntimeMetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub histograms: HashMap<String, Vec<u64>>,
}

impl RuntimeMetricsSnapshot {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            histograms: HashMap::new(),
        }
    }
}

impl Default for RuntimeMetricsSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Collector trait used by Runtime to record metrics. Implementations are
/// injected via the Builder and should be thread-safe.
pub trait RuntimeMetricsCollector: Send + Sync {
    fn incr(&self, key: &str, labels: Option<&[(&str, &str)]>);
    fn record_duration(&self, key: &str, dur: Duration, labels: Option<&[(&str, &str)]>);
    fn record_histogram(&self, key: &str, value: u64, labels: Option<&[(&str, &str)]>);
    fn snapshot(&self) -> RuntimeMetricsSnapshot;
}

/// Production-ready in-memory metrics collector. Uses atomic counters and
/// mutex-protected histograms. Labels are encoded into metric keys.
#[derive(Debug, Default)]
pub struct InMemoryMetricsCollector {
    counters: Mutex<HashMap<String, Arc<AtomicU64>>>,
    histograms: Mutex<HashMap<String, Vec<u64>>>,
    /// histogram buckets (millis) applied to all histograms
    buckets: Vec<u64>,
}

impl InMemoryMetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(HashMap::new()),
            histograms: Mutex::new(HashMap::new()),
            buckets: vec![10, 50, 100, 200, 500, 1000, 5000, 10000],
        }
    }

    fn key_with_labels(key: &str, labels: Option<&[(&str, &str)]>) -> String {
        if let Some(lbls) = labels {
            let mut s = String::from(key);
            for (k, v) in lbls {
                s.push('|');
                s.push_str(k);
                s.push('=');
                s.push_str(v);
            }
            s
        } else {
            key.to_string()
        }
    }
}

impl RuntimeMetricsCollector for InMemoryMetricsCollector {
    fn incr(&self, key: &str, labels: Option<&[(&str, &str)]>) {
        let k = Self::key_with_labels(key, labels);
        let mut guard = self.counters.lock().unwrap();
        let entry = guard
            .entry(k)
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        entry.fetch_add(1, Ordering::Relaxed);
    }

    fn record_duration(&self, key: &str, dur: Duration, labels: Option<&[(&str, &str)]>) {
        let ms = dur.as_millis() as u64;
        self.incr(&format!("{}_count", key), labels);
        // also push into histogram
        self.record_histogram(&format!("{}_hist", key), ms, labels);
    }

    fn record_histogram(&self, key: &str, value: u64, labels: Option<&[(&str, &str)]>) {
        let k = Self::key_with_labels(key, labels);
        let mut guard = self.histograms.lock().unwrap();
        let vec = guard
            .entry(k)
            .or_insert_with(|| vec![0u64; self.buckets.len() + 1]);
        // find bucket
        let mut placed = false;
        for (i, b) in self.buckets.iter().enumerate() {
            if value <= *b {
                vec[i] = vec[i].saturating_add(1);
                placed = true;
                break;
            }
        }
        if !placed {
            // last bucket is +Inf
            let last = vec.len() - 1;
            vec[last] = vec[last].saturating_add(1);
        }
    }

    fn snapshot(&self) -> RuntimeMetricsSnapshot {
        let mut snap = RuntimeMetricsSnapshot::new();
        let guard = self.counters.lock().unwrap();
        for (k, v) in guard.iter() {
            snap.counters.insert(k.clone(), v.load(Ordering::Relaxed));
        }
        let hguard = self.histograms.lock().unwrap();
        for (k, v) in hguard.iter() {
            snap.histograms.insert(k.clone(), v.clone());
        }
        snap
    }
}

/// Metrics collector wrapper that forwards runtime metrics to an optional
/// telemetry exporter while preserving the underlying collector semantics.
pub struct TelemetryMetricsCollector {
    inner: Arc<dyn RuntimeMetricsCollector>,
    exporter: Option<Arc<dyn crate::telemetry::TelemetryExporter>>,
}

impl TelemetryMetricsCollector {
    pub fn new(
        inner: Arc<dyn RuntimeMetricsCollector>,
        exporter: Option<Arc<dyn crate::telemetry::TelemetryExporter>>,
    ) -> Self {
        Self { inner, exporter }
    }
}

impl RuntimeMetricsCollector for TelemetryMetricsCollector {
    fn incr(&self, key: &str, labels: Option<&[(&str, &str)]>) {
        self.inner.incr(key, labels);
        if let Some(exporter) = &self.exporter {
            let _ = exporter.export_counter(key, 1, labels);
        }
    }

    fn record_duration(&self, key: &str, dur: Duration, labels: Option<&[(&str, &str)]>) {
        self.inner.record_duration(key, dur, labels);
        if let Some(exporter) = &self.exporter {
            let ms = dur.as_millis() as u64;
            let _ = exporter.export_counter(&format!("{}_count", key), 1, labels);
            let _ = exporter.export_histogram(&format!("{}_hist", key), &[ms], labels);
        }
    }

    fn record_histogram(&self, key: &str, value: u64, labels: Option<&[(&str, &str)]>) {
        self.inner.record_histogram(key, value, labels);
        if let Some(exporter) = &self.exporter {
            let _ = exporter.export_histogram(key, &[value], labels);
        }
    }

    fn snapshot(&self) -> RuntimeMetricsSnapshot {
        self.inner.snapshot()
    }
}

impl From<InMemoryMetricsCollector> for Arc<dyn RuntimeMetricsCollector> {
    fn from(v: InMemoryMetricsCollector) -> Self {
        Arc::new(v)
    }
}
