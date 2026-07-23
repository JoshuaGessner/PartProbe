# Cross-Platform Testing

> **Status:** Draft  
> **Last updated:** 2026-07-22  
> **Related requirements:** REQ-NF-001, REQ-NF-003, REQ-NF-006; TEST-008, TEST-012  
> **Related ADRs:** ADR-0001–ADR-0006  
> **Open questions:** Minimum OS/GPU versions and signing identities  
> **Dependencies:** Platform spikes  
> **Supersedes:** None

The CI matrix targets current supported Windows x86_64, macOS arm64/x86_64 as policy determines, and Linux x86_64. Core Rust tests must be identical. Native geometry, WebView, GPU, font, file-dialog, printing/PDF, HiDPI, path/Unicode, long-path, permission, backup, process-crash, package/install/uninstall, signing, and update behavior require platform-specific integration evidence.

Use deterministic headless tests where possible plus physical/virtual smoke systems. Record OS, architecture, GPU/backend, WebView/runtime, locale, scale factor, and package type with failures. A green core build does not prove packaging or viewport viability.
