# Workflow ECC Roadmap
## Raven AI Agent

Status: Design Phase

Target: Production Ready

Language: Rust

Architecture: Modular

Platform:
- Termux
- Linux
- Windows

---

# Tujuan

Workflow ECC bertanggung jawab menjaga integritas seluruh lifecycle workflow Raven.

Workflow ECC memastikan seluruh langkah workflow valid sebelum, selama, dan setelah dieksekusi.

Workflow ECC tidak menjalankan Tool.

Workflow ECC tidak membuat Planning.

Workflow ECC hanya memverifikasi bahwa workflow dapat dijalankan secara aman dan konsisten.

---

# Filosofi

Planner menentukan tujuan.

Tool menjalankan aksi.

Memory menyimpan hasil.

Workflow ECC memastikan seluruh proses tetap benar.

---

# Posisi Pada Arsitektur

User

↓

LLM

↓

Planner

↓

Planner ECC

↓

Workflow

↓

Workflow ECC

↓

Tool Manager

↓

Tool ECC

↓

Executor

↓

Executor ECC

↓

Memory

↓

Memory ECC

---

# Tanggung Jawab

Workflow ECC wajib mampu

- memvalidasi workflow
- memvalidasi dependency
- memvalidasi transisi state
- mendeteksi deadlock
- mendeteksi cycle
- mendeteksi orphan node
- mendeteksi workflow kosong
- mendeteksi step tidak valid
- mendeteksi step duplikat
- mendeteksi dependency yang hilang
- mendeteksi workflow tidak dapat diselesaikan
- melakukan recovery
- menghasilkan report
- menentukan policy
- menghitung confidence

---

# Struktur Folder

src/ecc/workflow/

mod.rs

builder.rs

engine.rs

pipeline.rs

validator.rs

corrector.rs

classifier.rs

confidence.rs

policy.rs

report.rs

context.rs

types.rs

rules.rs

errors.rs

tests/

---

# Pipeline

Workflow

↓

Validation

↓

Correction

↓

Classification

↓

Policy

↓

Confidence

↓

Reporting

↓

Verified Workflow

---

# Validator

Validator hanya mendeteksi.

Tidak memperbaiki.

Minimal memiliki rule berikut

WorkflowNotEmptyRule

UniqueStepIdRule

DependencyExistsRule

DependencyCycleRule

ReachabilityRule

TerminalStateRule

StartStateRule

EndStateRule

DuplicateStepRule

TransitionRule

TimeoutRule

RetryLimitRule

RollbackRule

CompensationRule

MaximumDepthRule

MaximumParallelRule

OrphanStepRule

DeadlockRule

DisconnectedGraphRule

---

# Corrector

Corrector hanya melakukan koreksi deterministik.

Contoh

menghapus dependency ganda

menormalkan status

mengurutkan dependency

menghapus edge kosong

menghapus node sementara

menormalkan identifier

Corrector tidak boleh

mengarang workflow

menambah step baru

mengubah tujuan workflow

mengubah planning

---

# Classification

Kategori minimal

Valid

MinorIssue

Recoverable

NonRecoverable

Corrupted

Deadlocked

Incomplete

---

# Policy

Decision

Accept

AcceptWithCorrection

Retry

Rollback

Abort

Quarantine

---

# Confidence

Rentang

0.0

↓

1.0

Dipengaruhi oleh

jumlah rule gagal

jumlah koreksi

jumlah dependency

jumlah recovery

tingkat kompleksitas workflow

---

# Reporting

WorkflowEccReport wajib berisi

ValidationReport

CorrectionReport

Classification

ConfidenceScore

ExecutedRules

AppliedFixes

RecoveryActions

PolicyDecision

Severity

ProcessingDuration

Timestamp

---

# Recovery

Workflow ECC wajib mendukung

Retry

Rollback

Resume

Skip Step

Abort

Recovery harus deterministik.

Tidak boleh menggunakan AI.

---

# Builder

WorkflowEccBuilder

register_validator()

register_rule()

register_corrector()

register_classifier()

register_policy()

register_confidence()

build()

---

# Trait

Validator

Corrector

Rule

Classifier

Reporter

ConfidenceScorer

Policy

PipelineStage

Recoverable

---

# Integrasi

WorkflowService

Planner

Executor

Memory

ToolManager

Semua workflow wajib melewati Workflow ECC.

---

# Unit Test

Setiap Rule memiliki test.

Setiap Validator memiliki test.

Setiap Corrector memiliki test.

Setiap Policy memiliki test.

Setiap Recovery memiliki test.

Pipeline memiliki test.

---

# Integration Test

Workflow valid

Workflow kosong

Duplicate Step

Dependency hilang

Dependency cycle

Deadlock

Rollback

Retry

Resume

Abort

Recovery

Confidence

Reporting

Policy

---

# Target Kualitas

cargo fmt

cargo check

cargo clippy --workspace --all-targets --all-features -- -D warnings

cargo test

cargo test --workspace

cargo doc --no-deps

Seluruhnya harus berhasil tanpa warning maupun error.

---

# Performa

Zero unsafe

Thread-safe

Send

Sync

Deterministic

Idempotent

Low Allocation

Reusable

Modular

Production Ready

---

# Larangan

Tidak boleh menggunakan

TODO

FIXME

placeholder

dummy

mock

stub

skeleton

prototype

hardcode

todo!()

unimplemented!()

panic!()

unwrap()

expect()

allow(dead_code)

allow(unused)

allow(clippy::*)

Implementasi harus nyata.

---

# Future

Distributed Workflow

Parallel Workflow Validation

Workflow Versioning

Workflow Snapshot

Workflow Replay

Workflow Audit

Workflow Visualization

Dynamic Workflow Policy

Workflow Optimization

Cross Workflow Validation

---

# Hubungan Dengan ECC Lain

Planner ECC

↓

menghasilkan execution plan

Workflow ECC

↓

memastikan execution plan dapat dijalankan

Tool ECC

↓

memastikan setiap tool valid

Executor ECC

↓

memastikan eksekusi aman

Memory ECC

↓

memastikan hasil tersimpan dengan benar

---

# Status

Design Complete

Belum diimplementasikan.

Implementasi dilakukan bertahap sesuai roadmap.