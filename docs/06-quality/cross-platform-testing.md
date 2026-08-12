# Cross-Platform Testing

> **Status:** Draft  
> **Last updated:** 2026-08-12
> **Related requirements:** REQ-NF-001, REQ-NF-003, REQ-NF-006, REQ-NF-018; TEST-008, TEST-011, TEST-012, TEST-019, TEST-023
> **Related ADRs:** ADR-0001–ADR-0006  
> **Open questions:** Minimum OS/GPU versions and signing identities  
> **Dependencies:** Platform spikes  
> **Supersedes:** None

The CI matrix targets current supported Windows x86_64, macOS arm64/x86_64 as policy determines, and Linux x86_64. Core Rust tests share one contract while retaining platform-specific evidence where primitives differ. Manual run 31555774851 adds Ubuntu 24.04 x86_64 exact OCCT construction, runtime assembly/host verification, internal dynamic-link closure, and configured desktop-host smoke evidence under the Linux parser filter. The Windows x64 workflow pins Visual Studio 2022/x64 plus canonical OCCT install directories, fingerprints runtime DLLs, keeps them app-local beside the worker, and requires x64 PE plus complete OCCT import-closure inspection before the same configured smoke. Run 31599859156 proves exact construction, canonical installation, and the DLL fingerprint but stopped when strict MSVC compilation exposed missing exception-unwind semantics and a deprecated diagnostic-copy call in the PartProbe bridge; the portability fix still requires a corrected rerun. Both native workflows upload no binaries and do not exercise a window or package. macOS hard memory and hostile-descendant escape prevention, macOS/Windows parser egress denial, Windows hard file-write bytes, general filesystem confinement, Linux aarch64 native behavior, completed Windows native evidence, and Linux GUI/package behavior remain explicitly unsupported or unproven rather than inferred from another platform's result. Native geometry, WebView, GPU, font, file-dialog, printing/PDF, HiDPI, path/Unicode, long-path, permission, backup, process-crash, package/install/uninstall, signing, and update behavior require platform-specific integration evidence.

Use deterministic headless tests where possible plus physical/virtual smoke systems. Record OS, architecture, GPU/backend, WebView/runtime, locale, scale factor, and package type with failures. A green core build does not prove packaging or viewport viability.
