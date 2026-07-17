# Tool ECC Roadmap
## Raven AI Agent

Status: Design Phase
Target: Production Ready
Language: Rust
Architecture: Modular
Compatibility: Termux / Linux / Windows

---

# Tujuan

Tool ECC adalah lapisan validasi deterministik yang berada di antara LLM dan Tool Executor.

Tujuannya bukan membuat Tool baru.

Tujuannya adalah memastikan seluruh output LLM yang akan menjalankan Tool telah valid, aman, konsisten, dan dapat dieksekusi.

Tool ECC harus mampu memperbaiki kesalahan kecil tanpa meminta LLM berpikir ulang.

---

# Filosofi

LLM menghasilkan kemungkinan.

ECC menghasilkan kepastian.

Executor hanya menerima output yang sudah lolos ECC.

---

# Posisi pada Arsitektur

User

↓

LLM

↓

Planner

↓

Planner ECC

↓

Tool Call

↓

Tool ECC

↓

Executor

↓

Workflow

↓

Memory ECC

---

# Target Kemampuan

Tool ECC wajib mampu:

- memvalidasi parameter
- memperbaiki parameter sederhana
- menghapus parameter ilegal
- mendeteksi tool yang salah
- mendeteksi tool yang tidak tersedia
- memverifikasi dependency tool
- melakukan scoring
- menghasilkan report
- menghasilkan confidence score
- mengembalikan keputusan Accept / Retry / Reject

---

# Struktur Folder

src/ecc/tool/

mod.rs

engine.rs

validator.rs

corrector.rs

policy.rs

pipeline.rs

report.rs

errors.rs

rules.rs

classifier.rs

confidence.rs

context.rs

builder.rs

tests/

---

# Pipeline

Input

↓

Validation Stage

↓

Correction Stage

↓

Classification Stage

↓

Policy Stage

↓

Confidence Stage

↓

Reporting Stage

↓

Executor

---

# Validator

Validator hanya bertugas mendeteksi masalah.

Tidak boleh memperbaiki.

Contoh:

- parameter kosong
- tipe salah
- field hilang
- field berlebih
- enum salah
- format JSON salah
- nama tool salah
- tool tidak terdaftar
- dependency tidak tersedia

Output:

ValidationReport

---

# Corrector

Corrector memperbaiki kesalahan yang bersifat deterministik.

Contoh:

String

↓

Integer

Boolean

↓

true

Nama Tool typo

↓

nama tool benar

Whitespace

↓

dibersihkan

Parameter duplicate

↓

dihapus

Tidak boleh melakukan inferensi.

Tidak boleh membuat parameter baru yang tidak diketahui.

---

# Rule Engine

Semua validator menggunakan Rule.

Trait

ToolRule

method:

validate()

Setiap rule independen.

Contoh rule

RequiredFieldRule

JsonFormatRule

UnknownToolRule

AllowedParameterRule

ParameterTypeRule

DependencyRule

DuplicateFieldRule

MaximumParameterRule

MinimumParameterRule

DangerousArgumentRule

ReservedFieldRule

PermissionRule

TimeoutRule

SandboxRule

---

# Error Classification

Critical

Executor tidak boleh berjalan.

High

Retry diperlukan.

Medium

ECC mencoba memperbaiki.

Low

Langsung diperbaiki.

---

# Policy

Policy menentukan keputusan akhir.

Accept

Retry

Reject

Escalate

---

# Confidence

Semua hasil memperoleh confidence.

0.0

↓

1.0

Contoh

Validation sukses

0.99

Auto correction

0.85

Retry

0.55

Reject

0.10

---

# Reporting

Setiap Tool ECC menghasilkan report.

Isi report

ValidationReport

CorrectionReport

ConfidenceScore

AppliedRules

AppliedFixes

ExecutionDecision

ProcessingTime

Severity

---

# Engine

Engine menghubungkan seluruh pipeline.

Validator

↓

Corrector

↓

Classifier

↓

Policy

↓

Confidence

↓

Reporter

---

# Builder

Builder membuat konfigurasi pipeline.

Contoh

ToolEccBuilder

↓

register_rule()

↓

register_validator()

↓

register_corrector()

↓

build()

---

# Trait

Validator

Corrector

Reporter

Rule

Policy

ConfidenceScorer

Classifier

PipelineStage

---

# Integrasi

Tahap pertama

ToolService

Tahap kedua

Executor

Tahap ketiga

Workflow

Tahap keempat

Memory

---

# Integrasi Tool Service

Sebelum Tool dijalankan

↓

Tool ECC

↓

Executor

Tidak boleh ada Tool yang melewati Tool ECC.

---

# Unit Test

Semua rule wajib memiliki test.

Semua corrector wajib memiliki test.

Semua policy wajib memiliki test.

Semua pipeline wajib memiliki test.

---

# Integration Test

Tool typo

Parameter typo

Parameter duplicate

JSON rusak

Dependency gagal

Retry

Reject

Accept

---

# Target Coverage

cargo test

100% lulus

cargo clippy

0 warning

cargo fmt

bersih

cargo doc

berhasil

---

# Design Rules

Tidak boleh menggunakan

todo!()

unimplemented!()

panic!()

unwrap()

expect()

mock

dummy

placeholder

stub

prototype

hardcoded

---

# Performance

Zero unsafe

Zero allocation tidak perlu

Deterministic

Thread-safe

Send

Sync

Idiomatic Rust

Modular

Production Ready

---

# Future

Schema Validator

JSON Schema

Semantic Validator

LLM Tool Repair

Dynamic Rule Loading

Plugin Tool Rules

Adaptive Confidence

Cross Tool Validation

Workflow Tool Verification

Memory Tool Verification

Distributed Tool ECC

---

Status

Design Complete

Belum diintegrasikan.

Implementasi dilakukan bertahap sesuai roadmap.