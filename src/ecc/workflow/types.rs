//! Type definitions for Workflow ECC.
//!
//! Mendefinisikan struktur data utama yang digunakan dalam validasi dan koreksi workflow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Workflow step atau aksi dalam eksekusi workflow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowStep {
    /// ID unik step.
    pub id: String,
    /// Deskripsi step.
    pub description: Option<String>,
    /// Timeout untuk step dalam milidetik.
    pub timeout_ms: Option<u64>,
    /// Jumlah retry maksimal.
    pub max_retries: Option<u32>,
    /// ID step yang harus diselesaikan sebelum step ini.
    pub dependencies: Vec<String>,
}

impl WorkflowStep {
    /// Buat step baru dengan ID.
    pub fn new(id: String) -> Self {
        Self {
            id,
            description: None,
            timeout_ms: None,
            max_retries: None,
            dependencies: Vec::new(),
        }
    }

    /// Set deskripsi step.
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    /// Set timeout dalam milidetik.
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set maksimal retry.
    pub fn with_retries(mut self, count: u32) -> Self {
        self.max_retries = Some(count);
        self
    }

    /// Tambahkan dependency.
    pub fn add_dependency(mut self, dep: String) -> Self {
        if !self.dependencies.contains(&dep) {
            self.dependencies.push(dep);
        }
        self
    }
}

/// Representasi workflow yang akan divalidasi dan diperbaiki.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// ID unik workflow.
    pub id: String,
    /// Deskripsi workflow.
    pub description: Option<String>,
    /// Timestamp pembuatan.
    pub created_at: DateTime<Utc>,
    /// Metadata tambahan.
    pub metadata: HashMap<String, String>,
    /// Daftar steps dalam workflow.
    pub steps: Vec<WorkflowStep>,
    /// ID step yang menjadi awal eksekusi.
    pub start_step_id: Option<String>,
    /// ID steps yang merupakan terminal state (workflow selesai).
    pub end_step_ids: Vec<String>,
    /// Timeout total untuk seluruh workflow dalam milidetik.
    pub total_timeout_ms: Option<u64>,
    /// Konfigurasi recovery.
    pub recovery_config: RecoveryConfig,
}

impl Workflow {
    /// Buat workflow baru dengan ID.
    pub fn new(id: String) -> Self {
        Self {
            id,
            description: None,
            created_at: Utc::now(),
            metadata: HashMap::new(),
            steps: Vec::new(),
            start_step_id: None,
            end_step_ids: Vec::new(),
            total_timeout_ms: None,
            recovery_config: RecoveryConfig::default(),
        }
    }

    /// Tambahkan step ke workflow.
    pub fn add_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Set start step.
    pub fn with_start_step(mut self, step_id: String) -> Self {
        self.start_step_id = Some(step_id);
        self
    }

    /// Tambahkan end step.
    pub fn add_end_step(mut self, step_id: String) -> Self {
        if !self.end_step_ids.contains(&step_id) {
            self.end_step_ids.push(step_id);
        }
        self
    }

    /// Dapatkan step berdasarkan ID.
    pub fn get_step(&self, id: &str) -> Option<&WorkflowStep> {
        self.steps.iter().find(|s| s.id == id)
    }

    /// Dapatkan ID semua steps.
    pub fn step_ids(&self) -> HashSet<String> {
        self.steps.iter().map(|s| s.id.clone()).collect()
    }

    /// Periksa apakah workflow kosong.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Periksa apakah workflow valid (minimal).
    pub fn has_valid_structure(&self) -> bool {
        !self.is_empty() && self.start_step_id.is_some() && !self.end_step_ids.is_empty()
    }
}

/// Konfigurasi recovery untuk workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Apakah retry diaktifkan.
    pub enable_retry: bool,
    /// Apakah rollback diaktifkan.
    pub enable_rollback: bool,
    /// Apakah resume diaktifkan.
    pub enable_resume: bool,
    /// Delay antara retry dalam milidetik.
    pub retry_delay_ms: Option<u64>,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enable_retry: true,
            enable_rollback: true,
            enable_resume: true,
            retry_delay_ms: Some(1000),
        }
    }
}

/// Opsi untuk analisis workflow.
#[derive(Debug, Clone)]
pub struct WorkflowAnalysisOptions {
    /// Apakah harus periksa deadlock.
    pub check_deadlock: bool,
    /// Apakah harus periksa cycle.
    pub check_cycle: bool,
    /// Apakah harus periksa reachability.
    pub check_reachability: bool,
    /// Apakah harus periksa orphan.
    pub check_orphan: bool,
    /// Depth maksimal untuk analisis graph.
    pub max_depth: Option<usize>,
}

impl Default for WorkflowAnalysisOptions {
    fn default() -> Self {
        Self {
            check_deadlock: true,
            check_cycle: true,
            check_reachability: true,
            check_orphan: true,
            max_depth: Some(1000),
        }
    }
}
