use crate::ecc::errors::EccResult;
use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;
use crate::planner::ExecutionPlan;

/// Rule: plan tidak boleh kosong.
pub struct PlanNotEmptyRule;

impl Rule<ExecutionPlan> for PlanNotEmptyRule {
    fn id(&self) -> &'static str {
        "structure.plan_not_empty"
    }

    fn description(&self) -> &'static str {
        "Plan must contain at least one step."
    }

    fn applies_to(&self, _plan: &ExecutionPlan) -> bool {
        true
    }

    fn evaluate(&self, plan: &ExecutionPlan) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if plan.steps.is_empty() {
            issues.push(EccIssue::new(
                self.id().into(),
                self.description().into(),
                Some("execution plan has no steps".into()),
                None,
            ));
        }
        Ok(issues)
    }
}

/// Rule: setiap langkah harus memiliki ID unik.
pub struct UniqueStepIdsRule;

impl Rule<ExecutionPlan> for UniqueStepIdsRule {
    fn id(&self) -> &'static str {
        "consistency.unique_step_ids"
    }

    fn description(&self) -> &'static str {
        "Each plan step must have a unique identifier."
    }

    fn applies_to(&self, _plan: &ExecutionPlan) -> bool {
        true
    }

    fn evaluate(&self, plan: &ExecutionPlan) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for step in &plan.steps {
            if !seen.insert(&step.id) {
                issues.push(EccIssue::new(
                    self.id().into(),
                    self.description().into(),
                    Some(format!("duplicate step id found: {}", step.id)),
                    Some(step.id.clone()),
                ));
            }
        }
        Ok(issues)
    }
}

/// Rule: setiap langkah harus memiliki ID yang tidak kosong.
pub struct StepIdNotEmptyRule;

impl Rule<ExecutionPlan> for StepIdNotEmptyRule {
    fn id(&self) -> &'static str {
        "consistency.step_id_not_empty"
    }

    fn description(&self) -> &'static str {
        "Each plan step must have a non-empty identifier."
    }

    fn applies_to(&self, _plan: &ExecutionPlan) -> bool {
        true
    }

    fn evaluate(&self, plan: &ExecutionPlan) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        for step in &plan.steps {
            if step.id.trim().is_empty() {
                issues.push(EccIssue::new(
                    self.id().into(),
                    self.description().into(),
                    Some(format!(
                        "step has an empty identifier: {}",
                        step.description
                    )),
                    Some(step.id.clone()),
                ));
            }
        }
        Ok(issues)
    }
}

/// Rule: dependencies harus mengarah ke langkah yang ada.
pub struct DependencyExistsRule;

impl Rule<ExecutionPlan> for DependencyExistsRule {
    fn id(&self) -> &'static str {
        "dependency.dependency_exists"
    }

    fn description(&self) -> &'static str {
        "Each dependency must reference an existing step."
    }

    fn applies_to(&self, plan: &ExecutionPlan) -> bool {
        !plan.steps.is_empty()
    }

    fn evaluate(&self, plan: &ExecutionPlan) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        let ids: std::collections::HashSet<&String> =
            plan.steps.iter().map(|step| &step.id).collect();
        for step in &plan.steps {
            for dep in &step.depends_on {
                if !ids.contains(dep) {
                    issues.push(EccIssue::new(
                        self.id().into(),
                        self.description().into(),
                        Some(format!("dependency {} references missing step", dep)),
                        Some(step.id.clone()),
                    ));
                }
            }
        }
        Ok(issues)
    }
}

/// Rule: duplikat dependency harus dideteksi oleh validator.
pub struct DuplicateDependencyRule;

impl Rule<ExecutionPlan> for DuplicateDependencyRule {
    fn id(&self) -> &'static str {
        "consistency.duplicate_dependency"
    }

    fn description(&self) -> &'static str {
        "Plan step dependencies should not contain duplicates."
    }

    fn applies_to(&self, plan: &ExecutionPlan) -> bool {
        !plan.steps.is_empty()
    }

    fn evaluate(&self, plan: &ExecutionPlan) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        for step in &plan.steps {
            let mut seen = std::collections::HashSet::new();
            for dep in &step.depends_on {
                if !seen.insert(dep) {
                    issues.push(EccIssue::new(
                        self.id().into(),
                        self.description().into(),
                        Some(format!(
                            "duplicate dependency {} found in step {}",
                            dep, step.id
                        )),
                        Some(step.id.clone()),
                    ));
                }
            }
        }
        Ok(issues)
    }
}

/// Rule: dependency tidak mengandung siklus.
pub struct AcyclicDependencyRule;

impl Rule<ExecutionPlan> for AcyclicDependencyRule {
    fn id(&self) -> &'static str {
        "dependency.acyclic"
    }

    fn description(&self) -> &'static str {
        "Plan dependencies must not form a cycle."
    }

    fn applies_to(&self, plan: &ExecutionPlan) -> bool {
        plan.steps.len() > 1
    }

    fn evaluate(&self, plan: &ExecutionPlan) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        let mut indeg = std::collections::HashMap::new();
        let mut graph: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for step in &plan.steps {
            indeg.insert(step.id.clone(), 0usize);
        }

        for step in &plan.steps {
            for dep in &step.depends_on {
                if let Some(entry) = indeg.get_mut(&step.id) {
                    *entry += 1;
                }
                graph.entry(dep.clone()).or_default().push(step.id.clone());
            }
        }

        let mut queue: std::collections::VecDeque<String> = indeg
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut visited = 0;
        while let Some(node) = queue.pop_front() {
            visited += 1;
            if let Some(neighbors) = graph.get(&node) {
                for neighbor in neighbors {
                    if let Some(count) = indeg.get_mut(neighbor) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        if visited != plan.steps.len() {
            issues.push(EccIssue::new(
                self.id().into(),
                self.description().into(),
                Some("dependency cycle detected".into()),
                None,
            ));
        }

        Ok(issues)
    }
}

/// Rule: semua langkah harus dapat dijangkau dari set entry.
pub struct ReachabilityRule;

impl Rule<ExecutionPlan> for ReachabilityRule {
    fn id(&self) -> &'static str {
        "reachability.all_steps_reachable"
    }

    fn description(&self) -> &'static str {
        "All steps must be reachable from at least one entry point."
    }

    fn applies_to(&self, plan: &ExecutionPlan) -> bool {
        !plan.steps.is_empty()
    }

    fn evaluate(&self, plan: &ExecutionPlan) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        let mut reachable = std::collections::HashSet::new();
        for step in &plan.steps {
            if step.depends_on.is_empty() {
                reachable.insert(step.id.clone());
            }
        }

        let mut added = true;
        while added {
            added = false;
            for step in &plan.steps {
                if !reachable.contains(&step.id)
                    && step.depends_on.iter().all(|dep| reachable.contains(dep))
                {
                    reachable.insert(step.id.clone());
                    added = true;
                }
            }
        }

        for step in &plan.steps {
            if !reachable.contains(&step.id) {
                issues.push(EccIssue::new(
                    self.id().into(),
                    self.description().into(),
                    Some(format!("step {} is unreachable", step.id)),
                    Some(step.id.clone()),
                ));
            }
        }

        Ok(issues)
    }
}

/// Rule: plan harus memiliki titik awal dan akhir yang valid.
pub struct PlanStartEndRule;

impl Rule<ExecutionPlan> for PlanStartEndRule {
    fn id(&self) -> &'static str {
        "structure.start_end_points"
    }

    fn description(&self) -> &'static str {
        "Plan must have valid start and end entry points."
    }

    fn applies_to(&self, plan: &ExecutionPlan) -> bool {
        !plan.steps.is_empty()
    }

    fn evaluate(&self, plan: &ExecutionPlan) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        let start_count = plan
            .steps
            .iter()
            .filter(|step| step.depends_on.is_empty())
            .count();
        let end_count = plan
            .steps
            .iter()
            .filter(|step| {
                !plan
                    .steps
                    .iter()
                    .any(|candidate| candidate.depends_on.contains(&step.id))
            })
            .count();

        if start_count == 0 {
            issues.push(EccIssue::new(
                self.id().into(),
                self.description().into(),
                Some("no start point found".into()),
                None,
            ));
        }
        if end_count == 0 {
            issues.push(EccIssue::new(
                self.id().into(),
                self.description().into(),
                Some("no end point found".into()),
                None,
            ));
        }
        Ok(issues)
    }
}
