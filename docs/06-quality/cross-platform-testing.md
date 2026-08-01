# Cross-Platform Testing

> **Status:** Draft  
> **Last updated:** 2026-08-01
> **Related requirements:** REQ-NF-001, REQ-NF-003, REQ-NF-006, REQ-NF-018; TEST-008, TEST-011, TEST-012, TEST-019, TEST-023
> **Related ADRs:** ADR-0001–ADR-0006  
> **Open questions:** Minimum OS/GPU versions and signing identities  
> **Dependencies:** Platform spikes  
> **Supersedes:** None

The CI matrix targets current supported Windows x86_64, macOS arm64/x86_64 as policy determines, and Linux x86_64. Core Rust tests must share one contract while retaining platform-specific evidence where primitives differ. Worker transport evidence records the actual mode per target: `verified_private_copy` is implemented everywhere; Unix `unix_descriptor` has an exact close-on-exec allowlist; Windows `windows_handle` uses an exact `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`; and both direct launchers prove an intentionally inheritable unrelated resource is excluded while preserving stdin/stdout, a cleared environment, controlled working directory, source verification, cancellation, forced termination, and reap behavior. Resource evidence records Unix CPU/file/process-group limits, Linux address-space limits, and Windows suspended Job assignment with CPU/memory/one-process/tree-kill controls. macOS hard memory, hostile Unix descendant escape prevention, and Windows hard file-write bytes remain explicitly unsupported rather than inferred from another platform's result. Native geometry, WebView, GPU, font, file-dialog, printing/PDF, HiDPI, path/Unicode, long-path, permission, backup, process-crash, package/install/uninstall, signing, and update behavior require platform-specific integration evidence.

Use deterministic headless tests where possible plus physical/virtual smoke systems. Record OS, architecture, GPU/backend, WebView/runtime, locale, scale factor, and package type with failures. A green core build does not prove packaging or viewport viability.
