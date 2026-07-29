# TASK-002 Validation Evidence

> **Status:** Complete
> **Last updated:** 2026-07-29
> **Related requirements:** REQ-F-008–REQ-F-010, REQ-F-014–REQ-F-018, REQ-F-032; REQ-NF-003, REQ-NF-004, REQ-NF-009, REQ-NF-017, REQ-NF-020; UX-011–UX-012; CALC-007–CALC-018; TEST-002, TEST-014
> **Related ADRs:** ADR-0007
> **Open questions:** Organization-specific cost categories, accounting treatment, pricing/rounding policy, roles, and calibration under TASK-007
> **Dependencies:** Rust 1.94.1, Cargo.lock
> **Supersedes:** None

## Implemented evidence

- Empty `RateCard` creation proves the production domain supplies no numeric rate defaults.
- Validated immutable rate-card/entry IDs and versions; ISO-style explicit currency; nonnegative exact amount; typed basis/category/composition/scope; owner; inclusive effective periods; `Draft`/`Reviewed`/`Approved`/`Retired`/`Superseded` governance with retained prior decisions; actor/time/reason events; and source provenance.
- Deterministic ordered-scope resolution selects exactly one approved effective entry, returns `Unavailable` when absent/unapproved/out of period, and returns `Blocked` for equally applicable conflicts. The replay trace pins selector ID/version and the complete ordered scope request.
- Versioned pricing and rounding policies preserve method, threshold, currency, boundary, scale, mode, unrounded value, rounded value, and policy identity/version.
- CALC-007 material cost, CALC-008 setup cost, exact-only CALC-009 setup amortization, CALC-010 cycle classification, CALC-011 run cost with duplicate/composite-charge rejection, CALC-012 operation cost, CALC-013 base internal cost, CALC-014 risk reserve, CALC-015 total internal cost, existing CALC-016/017 pricing methods, and CALC-018 pricing-policy application.
- Calculation snapshot values can pin the complete resolved rate/card/entry trace and governed rounding result.
- `golden_estimates.json` is explicitly marked `synthetic_test_only`; it supplies no production defaults and carries fixture author/reviewer evidence for deterministic engineering review only.

## Synthetic golden results

| Fixture | Base internal cost | Risk | Total cost | Formula/rounded price |
|---|---:|---:|---:|---:|
| EX-01 | USD 485.00 | USD 35.00 | USD 520.00 | USD 702.00 |
| EX-03 | USD 610.00 | USD 65.00 | USD 675.00 | USD 911.25 |
| EX-12 Q1 | USD 520.00 | USD 45.00 | USD 565.00 | USD 762.75 |
| EX-12 Q10 | USD 1,650.00 | USD 100.00 | USD 1,750.00 | USD 2,362.50 |
| EX-12 Q50 | USD 6,700.00 | USD 280.00 | USD 6,980.00 | USD 9,423.00 |
| EX-12 Q200 | USD 24,000.00 | USD 800.00 | USD 24,800.00 | USD 33,480.00 |

These values validate exact calculation mechanics and trace shape. They are not shop standards, market estimates, accounting advice, production defaults, or evidence of real quoting accuracy.

## Local acceptance run

Environment: Apple Silicon macOS; Rust 1.94.1 workspace.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace --all-targets --locked` | Pass: 41 runtime tests |
| `cargo test --workspace --doc --locked` | Pass: one compile-fail unit-safety doctest |
| `python3 scripts/check_planning.py` | Pass: 136 documentation files |

TASK-002 adds 18 runtime tests: eight domain invariants and ten estimation/rate-resolution/golden/replay tests.

## Remaining evidence and limits

- GitHub Actions run 30461694180 passes the complete Rust workflow on Windows, Linux, and macOS for commit `ac767e7`.
- No rate-entry UI, persistence repository/migration, CSV import, authorization service, or production estimate workflow was implemented.
- The synthetic fixture contract is schema version 2 after making owner and component/composite classification mandatory. No production records exist to migrate; lifecycle `prior_decisions` defaults empty when reading earlier spike JSON, while earlier fixtures must be explicitly upgraded rather than guessed.
- Governed rounding for nonterminating setup-unit division remains pending; the spike returns `RoundingRequired` instead of silently approximating.
- TASK-005 must validate UX-011/012 and TEST-012/014 rate setup, import-preview, conflict recovery, and selected-rate explanation.
- TASK-006 must validate immutable persistence, migration, backup/restore, and replay for rate and policy versions.
- TASK-007/M0.2 must validate real shop categories, cost allocation, pricing/rounding policy, roles, approval thresholds, and calibration before any accuracy or policy-fit claim.
- ADR-0007 remains In Review.
