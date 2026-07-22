use crate::error::{RavenError, RavenResult};
use crate::planner::{ExecutionPlan, Step};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Simple scheduler metrics captured per schedule() invocation.
#[derive(Debug, Clone, Default)]
pub struct SchedulerMetrics {
    pub last_latency: Option<Duration>,
    pub schedule_count: u64,
}

/// Scheduler performs DAG validation, computes topological layers (levels),
/// and returns a deterministic execution ordering that allows parallel execution
/// by grouping steps into levels. Steps in the same level have no inter-dependencies
/// and therefore can be executed in parallel by the engine if desired.
pub struct Scheduler {
    pub concurrency_limit: usize,
    pub metrics: Mutex<SchedulerMetrics>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            concurrency_limit: 1,
            metrics: Mutex::new(SchedulerMetrics::default()),
        }
    }

    pub fn with_concurrency(limit: usize) -> Self {
        Self {
            concurrency_limit: limit,
            metrics: Mutex::new(SchedulerMetrics::default()),
        }
    }

    /// Schedule returns a new ExecutionPlan ordered according to dependencies,
    /// topological layering and priority. Steps that can run in parallel are
    /// assigned the same layer; ordering within a layer is deterministic.
    pub fn schedule(&self, plan: &ExecutionPlan) -> RavenResult<ExecutionPlan> {
        let start = Instant::now();

        // Validate non-empty
        if plan.steps.is_empty() {
            return Err(RavenError::Planner("empty plan".into()));
        }

        // Build graph and node map
        let mut indeg: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut nodes: HashMap<String, Step> = HashMap::new();

        for step in &plan.steps {
            indeg.entry(step.id.clone()).or_insert(0);
            nodes.insert(step.id.clone(), step.clone());
        }

        for step in &plan.steps {
            for dep in &step.depends_on {
                if !nodes.contains_key(dep) {
                    return Err(RavenError::Planner(format!(
                        "missing dependency {} for step {}",
                        dep, step.id
                    )));
                }
                *indeg.entry(step.id.clone()).or_insert(0) += 1;
                adj.entry(dep.clone()).or_default().push(step.id.clone());
            }

            // Validate timeout/deadline semantics if present
            if let (Some(t), Some(d)) = (step.timeout_ms, step.deadline_ms) {
                if t > d {
                    return Err(RavenError::Planner(format!(
                        "step {} has timeout > deadline ({} > {})",
                        step.id, t, d
                    )));
                }
            }
        }

        // Kahn's algorithm to compute levels (topological layering)
        let mut q: VecDeque<String> = indeg
            .iter()
            .filter(|&(_, &d)| d == 0)
            .map(|(k, _)| k.clone())
            .collect();

        // maintain level mapping
        let mut level: HashMap<String, usize> = HashMap::new();
        for id in q.iter() {
            level.insert(id.clone(), 0usize);
        }

        let mut ordered: Vec<Step> = Vec::new();
        let mut processed = 0usize;

        while let Some(id) = q.pop_front() {
            let current_level = *level.get(&id).unwrap_or(&0usize);

            // push node into ordered but we will sort later to ensure deterministic ordering
            if let Some(s) = nodes.get(&id) {
                let mut s = s.clone();
                // annotate scheduling layer into metadata for downstream consumers
                s.metadata
                    .insert("schedule_layer".to_string(), current_level.to_string());
                ordered.push(s);
            }

            processed += 1;
            if let Some(neighbors) = adj.get(&id) {
                for n in neighbors {
                    if let Some(v) = indeg.get_mut(n) {
                        *v -= 1;
                        // set level of neighbor to max(existing, current + 1)
                        let next_level = current_level + 1;
                        let prev = level.get(n).cloned().unwrap_or(0usize);
                        if next_level > prev {
                            level.insert(n.clone(), next_level);
                        }
                        if *v == 0 {
                            q.push_back(n.clone());
                        }
                    }
                }
            }
        }

        if processed != nodes.len() {
            return Err(RavenError::Planner(
                "cycle detected in plan dependencies".into(),
            ));
        }

        // Deterministic ordering: sort by (layer asc, priority desc, estimated_cost asc, id)
        ordered.sort_by(|a, b| {
            let la = a
                .metadata
                .get("schedule_layer")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0usize);
            let lb = b
                .metadata
                .get("schedule_layer")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0usize);
            la.cmp(&lb)
                .then_with(|| b.priority.cmp(&a.priority))
                .then_with(|| a.estimated_cost.cmp(&b.estimated_cost))
                .then_with(|| a.id.cmp(&b.id))
        });

        // Update metrics
        let latency = start.elapsed();
        if let Ok(mut m) = self.metrics.lock() {
            m.last_latency = Some(latency);
            m.schedule_count += 1;
        }

        Ok(ExecutionPlan { steps: ordered })
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
