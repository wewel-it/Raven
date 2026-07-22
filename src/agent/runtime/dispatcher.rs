use crate::event::EventBus;
use crate::executor::Executor;
use crate::planner::ExecutionPlan;
use crate::tool::{registry::ToolRegistry, Tool, ToolError, ToolManagerService};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// Descriptor that holds executor implementation and its capabilities.
pub struct ExecutorDescriptor {
    pub executor: Arc<dyn Executor>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl ExecutorDescriptor {
    pub fn new(
        executor: Arc<dyn Executor>,
        capabilities: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            executor,
            capabilities,
            metadata,
        }
    }
}

/// Dispatcher is a thread-safe registry for executors and tools.
/// It selects the best executor based on plan metadata and registered capabilities.
pub struct Dispatcher {
    executors: Arc<RwLock<HashMap<String, ExecutorDescriptor>>>,
    pub tools: Option<Arc<Mutex<Box<dyn ToolManagerService>>>>,
    pub tool_registry: Arc<ToolRegistry>,
    pub event_bus: Option<Arc<EventBus>>,
    pub metrics: Option<Arc<dyn crate::agent::runtime::metrics::RuntimeMetricsCollector>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            executors: Arc::new(RwLock::new(HashMap::new())),
            tools: None,
            tool_registry: Arc::new(ToolRegistry::new()),
            event_bus: None,
            metrics: None,
        }
    }

    pub fn with_tools(mut self, tools: Arc<Mutex<Box<dyn ToolManagerService>>>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = registry;
        self
    }

    pub fn with_event_bus(mut self, bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    pub fn with_metrics(
        mut self,
        metrics: Arc<dyn crate::agent::runtime::metrics::RuntimeMetricsCollector>,
    ) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Register an executor under a name, optionally with capabilities and metadata.
    pub fn register_executor_with_metadata(
        &self,
        name: impl Into<String>,
        exe: Arc<dyn Executor>,
        capabilities: Vec<String>,
        metadata: HashMap<String, String>,
    ) {
        let descriptor = ExecutorDescriptor::new(exe, capabilities, metadata);
        if let Ok(mut guard) = self.executors.write() {
            guard.insert(name.into(), descriptor);
            if let Some(m) = &self.metrics {
                m.incr("dispatcher_register_executor", None);
            }
        }
    }

    /// Unregister an executor by name.
    pub fn unregister_executor(&self, name: &str) {
        if let Ok(mut guard) = self.executors.write() {
            guard.remove(name);
        }
    }

    /// Register an executor under a name. Overwrites existing with same name.
    pub fn register_executor(&self, name: impl Into<String>, exe: Arc<dyn Executor>) {
        self.register_executor_with_metadata(name, exe, Vec::new(), HashMap::new());
    }

    /// Get executor by name.
    pub fn get_executor(&self, name: &str) -> Option<Arc<dyn Executor>> {
        if let Ok(guard) = self.executors.read() {
            guard
                .get(name)
                .map(|descriptor| descriptor.executor.clone())
        } else {
            None
        }
    }

    /// Resolve an executor by matching capabilities/metadata against a predicate.
    /// This provides a flexible lookup that avoids hardcoded if/else chains.
    pub fn resolve_executor<F>(&self, matcher: F) -> Option<Arc<dyn Executor>>
    where
        F: Fn(&ExecutorDescriptor) -> bool,
    {
        if let Ok(guard) = self.executors.read() {
            // deterministic ordering by key to keep selection stable
            let mut keys: Vec<_> = guard.keys().cloned().collect();
            keys.sort();
            for k in keys {
                if let Some(desc) = guard.get(&k) {
                    if matcher(desc) {
                        return Some(desc.executor.clone());
                    }
                }
            }
        }
        None
    }

    /// Register a tool in the dispatcher tool registry.
    pub fn register_tool(&self, tool: Arc<dyn Tool>) -> Result<(), ToolError> {
        self.tool_registry.register(tool)
    }

    /// Get a tool from the dispatcher tool registry.
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tool_registry.get(name).ok()
    }

    /// Dispatch the best executor for a given plan.
    pub fn dispatch(&self, plan: &ExecutionPlan) -> Option<Arc<dyn Executor>> {
        self.select_executor(plan)
    }

    /// Select an executor for a given plan. Selection is based on step metadata and capabilities.
    pub fn select_executor(&self, plan: &ExecutionPlan) -> Option<Arc<dyn Executor>> {
        // Preferred selection by explicit tool_name of first step.
        if let Some(first) = plan.steps.first() {
            if let Some(tool_name) = &first.tool_name {
                if let Some(executor) = self.get_executor(tool_name) {
                    if let Some(m) = &self.metrics {
                        m.incr(
                            "executor_selected",
                            Some(&[("executor", tool_name.as_str())]),
                        );
                    }
                    return Some(executor);
                }
            }
        }

        // Build a list of candidate descriptors and score them deterministically.
        if let Some(first) = plan.steps.first() {
            let required = first
                .metadata
                .get("executor")
                .map(String::as_str)
                .or(first.tool_name.as_deref());

            if let Ok(guard) = self.executors.read() {
                // deterministic iteration order
                let mut items: Vec<(&String, &ExecutorDescriptor)> = guard.iter().collect();
                items.sort_by_key(|(k, _)| (*k).clone());

                // compute best match by simple scoring: metadata match > capability match > priority fallback
                let mut best: Option<(&String, &ExecutorDescriptor, i32)> = None;
                for (name, desc) in items {
                    let mut score: i32 = 0;
                    if let Some(req) = required {
                        if desc.metadata.get("executor").map(String::as_str) == Some(req) {
                            score += 100;
                        }
                        if desc.capabilities.iter().any(|c| c == req) {
                            score += 50;
                        }
                    }
                    if let Some(tool_name) = &first.tool_name {
                        if desc.capabilities.iter().any(|c| c == tool_name) {
                            score += 40;
                        }
                    }

                    // prefer executors that explicitly declare capabilities
                    if !desc.capabilities.is_empty() {
                        score += 1;
                    }

                    if let Some((_, _, best_score)) = &best {
                        if score > *best_score {
                            best = Some((name, desc, score));
                        }
                    } else {
                        best = Some((name, desc, score));
                    }
                }

                if let Some((name, desc, _)) = best {
                    if let Some(m) = &self.metrics {
                        m.incr("executor_resolved", Some(&[("executor", name.as_str())]));
                    }
                    return Some(desc.executor.clone());
                }
            }
        }

        // If only one executor is registered, return it as a fallback.
        if let Ok(guard) = self.executors.read() {
            if guard.len() == 1 {
                return guard
                    .values()
                    .next()
                    .map(|descriptor| descriptor.executor.clone());
            }
        }

        None
    }

    /// List names of registered executors.
    pub fn registered_executors(&self) -> Vec<String> {
        if let Ok(guard) = self.executors.read() {
            guard.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}
