use crate::error::{RavenError, RavenResult};
use crate::intent::Intent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

/// The status of a step during planning and execution tracking.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Retrying,
    Skipped,
}

/// Retry policy for a step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub attempts: u32,
    /// deterministic backoff multiplier (no sleeping here)
    pub backoff_multiplier: u32,
}

impl RetryPolicy {
    pub fn new() -> Self {
        Self {
            max_attempts: 3,
            attempts: 0,
            backoff_multiplier: 1,
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy::new()
    }
}

/// Single step in the plan. Designed to be extensible.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Step {
    pub id: String,
    pub description: String,
    pub needs_tool: bool,
    pub tool_name: Option<String>,
    pub params: Option<serde_json::Value>,
    pub depends_on: Vec<String>,
    pub estimated_cost: u32,
    pub priority: u8,
    pub status: StepStatus,
    pub last_update: Option<DateTime<Utc>>,
    pub retry_policy: RetryPolicy,
    pub attempt_log: Vec<String>,
}

impl Step {
    pub fn new(id: String, description: String) -> Self {
        Self {
            id,
            description,
            needs_tool: false,
            tool_name: None,
            params: None,
            depends_on: Vec::new(),
            estimated_cost: 1,
            priority: 5,
            status: StepStatus::Pending,
            last_update: None,
            retry_policy: RetryPolicy::new(),
            attempt_log: Vec::new(),
        }
    }
}

/// ExecutionPlan holds steps and provides helper APIs for inspection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub steps: Vec<Step>,
}

impl ExecutionPlan {
    pub fn empty() -> Self {
        ExecutionPlan { steps: Vec::new() }
    }

    pub fn find_step_index(&self, id: &str) -> Option<usize> {
        self.steps.iter().position(|s| s.id == id)
    }

    pub fn get_step(&self, id: &str) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }
}

/// Planner trait defines the Planner Service interface.
pub trait Planner: Send + Sync {
    fn analyze_goal(&self, goal: &str) -> RavenResult<serde_json::Value>;
    fn decompose(&self, goal: &serde_json::Value) -> RavenResult<ExecutionPlan>;
    fn resolve_dependencies(&self, plan: &mut ExecutionPlan) -> RavenResult<()>;
    fn create_plan(&self, intent: &Intent) -> RavenResult<ExecutionPlan>;
    fn replan_on_failure(&self, plan: &mut ExecutionPlan, failed_step_id: &str) -> RavenResult<()>;
}

/// Progress tracking interface used by the Executor to update step state
pub trait PlannerProgress: Send + Sync {
    fn mark_step_started(&self, id: &str) -> RavenResult<()>;
    fn mark_step_completed(&self, id: &str) -> RavenResult<()>;
    fn mark_step_failed(&self, id: &str, reason: &str) -> RavenResult<()>;
    fn should_retry(&self, id: &str) -> RavenResult<bool>;
    fn next_retry_backoff(&self, id: &str) -> RavenResult<(u32, u32)>;
    fn replan_on_failure(&self, plan: &mut ExecutionPlan, failed_step_id: &str) -> RavenResult<()>;
}

/// PlannerService: a deterministic, thread-safe planner implementation.
pub struct PlannerService {
    // configuration or heuristics can be stored here
    pub name: String,
    inner: Arc<RwLock<ExecutionPlan>>, // holds the current working plan for progress tracking
}

impl PlannerService {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            inner: Arc::new(RwLock::new(ExecutionPlan::empty())),
        }
    }

    /// Expose create_plan as an inherent method for easier usage.
    pub fn create_plan(&self, intent: &Intent) -> RavenResult<ExecutionPlan> {
        Planner::create_plan(self, intent)
    }

    /// Access a read-only snapshot of the current plan
    pub fn snapshot(&self) -> RavenResult<ExecutionPlan> {
        let guard = self
            .inner
            .read()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        Ok(guard.clone())
    }

    /// Update internal plan reference (used when plan is created or replanned)
    fn set_plan(&self, plan: ExecutionPlan) -> RavenResult<()> {
        let mut w = self
            .inner
            .write()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        *w = plan;
        Ok(())
    }

    /// Progress tracking helpers
    pub fn mark_step_started(&self, id: &str) -> RavenResult<()> {
        let mut w = self
            .inner
            .write()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        if let Some(idx) = w.find_step_index(id) {
            let s = &mut w.steps[idx];
            s.status = StepStatus::InProgress;
            s.last_update = Some(Utc::now());
            s.attempt_log.push(format!("started at {}", Utc::now()));
            Ok(())
        } else {
            Err(RavenError::Planner(format!("step not found: {}", id)))
        }
    }

    pub fn mark_step_completed(&self, id: &str) -> RavenResult<()> {
        let mut w = self
            .inner
            .write()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        if let Some(idx) = w.find_step_index(id) {
            let s = &mut w.steps[idx];
            s.status = StepStatus::Completed;
            s.last_update = Some(Utc::now());
            s.attempt_log.push(format!("completed at {}", Utc::now()));
            Ok(())
        } else {
            Err(RavenError::Planner(format!("step not found: {}", id)))
        }
    }

    pub fn mark_step_failed(&self, id: &str, reason: &str) -> RavenResult<()> {
        let mut w = self
            .inner
            .write()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        if let Some(idx) = w.find_step_index(id) {
            let s = &mut w.steps[idx];
            s.status = StepStatus::Failed;
            s.last_update = Some(Utc::now());
            s.retry_policy.attempts += 1;
            s.attempt_log
                .push(format!("failed at {}: {}", Utc::now(), reason));
            Ok(())
        } else {
            Err(RavenError::Planner(format!("step not found: {}", id)))
        }
    }

    pub fn should_retry(&self, id: &str) -> RavenResult<bool> {
        let w = self
            .inner
            .read()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        if let Some(s) = w.get_step(id) {
            Ok(s.retry_policy.attempts < s.retry_policy.max_attempts)
        } else {
            Err(RavenError::Planner(format!("step not found: {}", id)))
        }
    }

    /// Apply a deterministic retry scheduling decision (returns attempt number and backoff ticks)
    pub fn next_retry_backoff(&self, id: &str) -> RavenResult<(u32, u32)> {
        let w = self
            .inner
            .read()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        if let Some(s) = w.get_step(id) {
            let attempts = s.retry_policy.attempts;
            let backoff = (attempts + 1) * s.retry_policy.backoff_multiplier;
            Ok((attempts + 1, backoff))
        } else {
            Err(RavenError::Planner(format!("step not found: {}", id)))
        }
    }
}

impl Planner for PlannerService {
    /// Analyze a goal string into structured representation.
    /// Deterministic analysis: extracts sentences, keywords, and simple intent-like metadata.
    fn analyze_goal(&self, goal: &str) -> RavenResult<serde_json::Value> {
        if goal.trim().is_empty() {
            return Err(RavenError::Planner("empty goal".into()));
        }
        let mut obj = serde_json::Map::new();
        obj.insert(
            "raw".to_string(),
            serde_json::Value::String(goal.to_string()),
        );
        // decompose into sentences deterministically
        let parts: Vec<String> = goal
            .split(|c: char| ['.', '?', '!'].contains(&c))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        obj.insert(
            "sentences".to_string(),
            serde_json::Value::Array(
                parts
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            ),
        );

        // keywords heuristic
        let lower = goal.to_lowercase();
        let mut keywords = Vec::new();
        for kw in [
            "summarize",
            "plan",
            "execute",
            "run",
            "save",
            "search",
            "analyze",
            "build",
        ]
        .iter()
        {
            if lower.contains(kw) {
                keywords.push((*kw).to_string());
            }
        }
        obj.insert(
            "keywords".to_string(),
            serde_json::Value::Array(
                keywords
                    .iter()
                    .map(|k| serde_json::Value::String(k.clone()))
                    .collect(),
            ),
        );

        Ok(serde_json::Value::Object(obj))
    }

    /// Decompose structured goal into an ExecutionPlan deterministically.
    fn decompose(&self, goal: &serde_json::Value) -> RavenResult<ExecutionPlan> {
        let mut plan = ExecutionPlan::empty();
        let mut id_counter: usize = 1;
        // read sentences
        if let Some(sentences) = goal.get("sentences").and_then(|v| v.as_array()) {
            for s in sentences {
                if let Some(text) = s.as_str() {
                    let id = format!("step_{:04}", id_counter);
                    id_counter += 1;
                    let mut step = Step::new(id.clone(), text.to_string());
                    // Heuristics: mark as tool if sentence contains imperative verbs or 'run'/'execute'
                    let low = text.to_lowercase();
                    if low.contains("run")
                        || low.contains("execute")
                        || low.contains("call")
                        || low.contains("tool")
                    {
                        step.needs_tool = true;
                        // try to extract tool name pattern name=foo
                        if let Some(eq_idx) = text.find('=') {
                            let after = &text[eq_idx + 1..];
                            let token = after.split_whitespace().next().unwrap_or("");
                            if !token.is_empty() {
                                step.tool_name = Some(token.to_string());
                            }
                        }
                    }
                    // estimated cost: length-based deterministic metric
                    step.estimated_cost = (text.len() as u32).max(1) / 20 + 1;
                    // priority: presence of keywords increases priority
                    step.priority = if low.contains("important") { 9 } else { 5 };
                    plan.steps.push(step);
                }
            }
        }

        // If plan empty, create a single step from raw
        if plan.steps.is_empty() {
            if let Some(raw) = goal.get("raw").and_then(|v| v.as_str()) {
                let mut step = Step::new("step_0001".to_string(), raw.to_string());
                step.estimated_cost = (raw.len() as u32).max(1) / 20 + 1;
                plan.steps.push(step);
            }
        }

        // Resolve dependencies deterministically: create chain unless sentences contain 'and' which indicates parallelizable
        self.resolve_dependencies(&mut plan)?;
        Ok(plan)
    }

    /// Resolve dependencies deterministically. We link steps sequentially unless
    /// the sentence explicitly contains 'and' or ',' allowing parallel execution.
    fn resolve_dependencies(&self, plan: &mut ExecutionPlan) -> RavenResult<()> {
        for i in 0..plan.steps.len() {
            plan.steps[i].depends_on.clear();
        }
        for i in 1..plan.steps.len() {
            // clone prev id first to avoid overlapping borrows
            let prev_id = plan.steps[i - 1].id.clone();
            let cur = &mut plan.steps[i];
            let low = cur.description.to_lowercase();
            if low.contains(" and ") || low.contains(',') {
                // allow parallel: do not depend on previous
                continue;
            }
            // by default, current depends on previous
            cur.depends_on.push(prev_id);
        }

        // Validate no cycles via simple check using Kahn's algorithm on ids
        let ids: Vec<String> = plan.steps.iter().map(|s| s.id.clone()).collect();
        let mut indeg: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        for id in &ids {
            indeg.insert(id.clone(), 0usize);
        }
        for s in &plan.steps {
            for dep in &s.depends_on {
                if !indeg.contains_key(dep) {
                    return Err(RavenError::Planner(format!(
                        "unknown dependency {} on step {}",
                        dep, s.id
                    )));
                }
                if let Some(v) = indeg.get_mut(&s.id) {
                    *v += 1;
                } else {
                    return Err(RavenError::Planner(format!(
                        "internal error updating indegree for {}",
                        s.id
                    )));
                }
                graph.entry(dep.clone()).or_default().push(s.id.clone());
            }
        }
        let mut q: VecDeque<String> = indeg
            .iter()
            .filter(|&(_, &d)| d == 0)
            .map(|(k, _)| k.clone())
            .collect();
        let mut visited = 0usize;
        while let Some(id) = q.pop_front() {
            visited += 1;
            if let Some(neigh) = graph.get(&id) {
                for n in neigh {
                    if let Some(d) = indeg.get_mut(n) {
                        *d -= 1;
                        if *d == 0 {
                            q.push_back(n.clone());
                        }
                    }
                }
            }
        }
        if visited != ids.len() {
            return Err(RavenError::Planner("cyclic dependencies detected".into()));
        }
        Ok(())
    }

    /// High-level: create a full execution plan from Intent deterministically.
    fn create_plan(&self, intent: &Intent) -> RavenResult<ExecutionPlan> {
        // Use intent.raw as goal for the planner
        let goal_val = self.analyze_goal(&intent.raw)?;
        let mut plan = self.decompose(&goal_val)?;
        // set some deterministic ids and defaults
        for (i, step) in plan.steps.iter_mut().enumerate() {
            if step.id.is_empty() {
                step.id = format!("step_{:04}", i + 1);
            }
            step.last_update = Some(Utc::now());
        }
        // write to internal plan store for progress tracking
        self.set_plan(plan.clone())?;
        Ok(plan)
    }

    /// Replanning strategy triggered on step failure. Deterministic policy:
    /// - If step can be retried (attempts < max), mark as Retrying and keep plan.
    /// - If exceeded retries and step is critical (priority >=8), mark dependents Skipped.
    /// - Otherwise, mark step Skipped and continue.
    fn replan_on_failure(&self, plan: &mut ExecutionPlan, failed_step_id: &str) -> RavenResult<()> {
        // find step
        let idx = plan
            .find_step_index(failed_step_id)
            .ok_or_else(|| RavenError::Planner(format!("step {} not found", failed_step_id)))?;
        let step = &mut plan.steps[idx];
        step.status = StepStatus::Failed;
        step.last_update = Some(Utc::now());

        if step.retry_policy.attempts < step.retry_policy.max_attempts {
            step.status = StepStatus::Retrying;
            step.attempt_log.push(format!(
                "scheduled retry #{}",
                step.retry_policy.attempts + 1
            ));
            return Ok(());
        }

        // exceeded retries
        if step.priority >= 8 {
            // mark dependents skipped deterministically
            let failed_id = step.id.clone();
            for s in plan.steps.iter_mut() {
                if s.depends_on.contains(&failed_id) {
                    s.status = StepStatus::Skipped;
                    s.attempt_log
                        .push(format!("skipped due to failed dependency {}", failed_id));
                }
            }
        } else {
            // non-critical: mark as skipped and allow dependents to continue if they don't require it
            step.status = StepStatus::Skipped;
        }
        Ok(())
    }
}

impl PlannerProgress for PlannerService {
    fn mark_step_started(&self, id: &str) -> RavenResult<()> {
        PlannerService::mark_step_started(self, id)
    }
    fn mark_step_completed(&self, id: &str) -> RavenResult<()> {
        PlannerService::mark_step_completed(self, id)
    }
    fn mark_step_failed(&self, id: &str, reason: &str) -> RavenResult<()> {
        PlannerService::mark_step_failed(self, id, reason)
    }
    fn should_retry(&self, id: &str) -> RavenResult<bool> {
        PlannerService::should_retry(self, id)
    }
    fn next_retry_backoff(&self, id: &str) -> RavenResult<(u32, u32)> {
        PlannerService::next_retry_backoff(self, id)
    }
    fn replan_on_failure(&self, plan: &mut ExecutionPlan, failed_step_id: &str) -> RavenResult<()> {
        Planner::replan_on_failure(self, plan, failed_step_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::IntentAnalyzer;

    #[test]
    fn planner_service_creates_and_tracks_plan() {
        let analyzer = IntentAnalyzer::new();
        let intent_res =
            analyzer.analyze("Do step A. Then run tool name=echo. Finally summarize results.");
        let intent = match intent_res {
            Ok(i) => i,
            Err(_) => return,
        };
        let svc = PlannerService::new("test-planner");
        let plan_res = svc.create_plan(&intent);
        let plan = match plan_res {
            Ok(p) => p,
            Err(_) => return,
        };
        assert!(plan.steps.len() >= 3);
        // initial statuses are pending
        for s in &plan.steps {
            assert_eq!(s.status, StepStatus::Pending);
        }
        // mark first step started/completed
        assert!(svc.mark_step_started(&plan.steps[0].id).is_ok());
        assert!(svc.mark_step_completed(&plan.steps[0].id).is_ok());
        let snap = match svc.snapshot() {
            Ok(s) => s,
            Err(_) => return,
        };
        assert_eq!(snap.steps[0].status, StepStatus::Completed);
    }

    #[test]
    fn planner_replans_on_failure_and_retries() {
        let analyzer = IntentAnalyzer::new();
        let intent_res = analyzer.analyze("Run tool name=echo. Then do cleanup.");
        let intent = match intent_res {
            Ok(i) => i,
            Err(_) => return,
        };
        let svc = PlannerService::new("test-planner");
        let plan_res = svc.create_plan(&intent);
        let mut plan = match plan_res {
            Ok(p) => p,
            Err(_) => return,
        };
        // simulate failure on first step
        let failed_id = plan.steps[0].id.clone();
        // ensure retry policy small for test
        plan.steps[0].retry_policy.max_attempts = 1;
        assert!(Planner::replan_on_failure(&svc, &mut plan, &failed_id).is_ok());
        assert!(
            plan.steps[0].status == StepStatus::Retrying
                || plan.steps[0].status == StepStatus::Skipped
                || plan.steps[1].status == StepStatus::Skipped
        );
    }
}
