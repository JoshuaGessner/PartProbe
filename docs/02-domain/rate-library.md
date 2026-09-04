# Rate Library and Cost Policy

> **Status:** In Review
> **Last updated:** 2026-09-04
> **Related requirements:** REQ-F-008–REQ-F-010, REQ-F-015–REQ-F-018, REQ-F-032; REQ-NF-003, REQ-NF-009; CALC-007–CALC-018; DATA-005–DATA-008, DATA-011, DATA-017; TEST-002, TEST-006, TEST-007, TEST-014
> **Related ADRs:** ADR-0006, ADR-0007
> **Open questions:** OQ-012–OQ-018
> **Dependencies:** Calculation and persistence spikes; later shop calibration
> **Supersedes:** None

PartProbe ships with no production rate values. An organization creates and governs its own rate cards; synthetic values are allowed only in clearly isolated test or demonstration data and never become production defaults. The ordinary arbitrary-model workflow must not load a fixed synthetic estimate template and present the result as model-derived.

Reusable rate and pricing administration belongs in first-run setup and Settings, not in the primary estimate form. An estimate may show the selected version, readiness, conflict/staleness state, and a direct correction link, but it must not duplicate the library editor. The current session-only developer settings remain provisional until TASK-006 persistence, approval, migration, and replay evidence pass.

## Separate configuration layers

1. **Operational rates:** machine, setup labor, run labor, programming, inspection, burden, tooling, consumables, fixture, administration, and other explicitly named effort.
2. **Commercial offers:** material, outside processing, freight, certificates, vendor minimums, and other time-bounded sourced charges.
3. **Cost-allocation policies:** direct, indirect, fixed, variable, recurring, nonrecurring, and overhead treatment.
4. **Pricing policies:** markup or target margin, floors, minimum charges, customer adjustments, discounts, expedite charges, and approval thresholds.

Capability, financial rate, commercial offer, accounting cost, risk reserve, opportunity cost, contribution, and selling price remain distinct types and records. A composite workcenter rate and its component rates cannot both charge the same occurrence.

## Rate entry contract

Every reusable rate records:

- stable rate and rate-card IDs with immutable versions, plus an explicit `Component` or `Composite` composition that prevents bundled and atomic charges from being combined;
- nonnegative exact-decimal amount, explicit three-letter currency code, and typed basis such as hour, minute, item, lot, setup, mass, length, or flat charge; the production application boundary validates the code against a pinned ISO 4217 registry version, while the current spike validates its uppercase three-letter form;
- one declared applicability scope such as organization, site, cost center, workcenter, machine, operation, labor class, material, or vendor;
- effective-from and optional effective-through dates;
- source, owner, entered-at identity/time, retained prior decisions, current review or approval identity/time/reason, and exact lifecycle state: `Draft`, `Reviewed`, `Approved`, `Retired`, or `Superseded`;
- optional evidence and notes held outside calculation authority.

Effective intervals are inclusive of both `effective_from` and an optional `effective_through`. Approved versions are immutable. A correction creates a new version and effective interval. Records referenced by an estimate are retired or superseded, never destructively deleted.

## Selection and missing-data behavior

The estimate may explicitly select an approved rate or request deterministic resolution from an ordered, caller-declared scope list. The first scope with exactly one approved effective match wins; zero matches continue to the next declared scope, while multiple matches at one rank block resolution. The calculation trace records the selector ID/version, complete ordered scope request, selected entry, rate-card version, effective date, selection rank, and reason.

- No applicable approved rate yields `Unavailable`, never zero.
- More than one equally applicable approved rate yields `Blocked`.
- Expired, future, draft, reviewed, retired, and superseded entries are never silently treated as current approved values.
- Currency or basis mismatch is an error; mixed-currency totals require a separate explicit conversion policy and snapshot.
- Estimate-specific overrides preserve the original value, new value, actor, time, reason, and authorization.

## Currency and rounding

Organization currency is explicitly selected and never inferred from locale. Initial setup suggests USD because it is the expected common case, while keeping the field editable and requiring explicit confirmation before rate entry. Calculations retain exact decimal values until a named rounding boundary. A versioned rounding policy declares currency, scale or increment, mode, and boundary: supplier charge, line item, quote total, or presentation only. Initial setup proposes the confirmed currency minor unit and half-even presentation rounding, but requires confirmation and stores the resulting organization policy. Each rounded result preserves the unrounded value, rounded value, policy version, scale/increment, mode, and boundary. Historical estimates pin the rate, selector, pricing, rule, and rounding-policy versions used.

The current spike evaluates setup-unit amortization only when division is exact. A nonterminating result returns `RoundingRequired`; applying a named line/presentation rounding policy to rational division remains an explicit follow-on before production display.

## Entry and review UX

The initial rate setup experience provides a guided category checklist, compact editable grid, explicit unit/currency suffixes, effective-date controls, source and approval state, formula preview, missing/overlap warnings, duplicate-as-new-version, and CSV/bulk-paste staging with row-level validation. Templates may provide structure and category names but no numeric production defaults.

The estimate cost trace shows the selected rate, why it applied, its age and approval state, the unrounded extension, and any rounding boundary. Basic rate entry is required for the initial vertical slice; advanced multi-user library governance may follow after persistence and authorization evidence.

## Validation and calibration

TASK-002 uses synthetic, test-only rate cards for exact calculation and replay evidence. These fixtures prove software behavior but are not shop standards and do not satisfy real-world accuracy claims. TASK-007 and M0.2 retain responsibility for shop rate-category review, accounting/pricing-policy validation, role/approval decisions, and representative calibration.

## Design references

- [ISO 4217 currency codes](https://www.iso.org/iso-4217-currency-codes.html) for explicit currency identity and minor-unit relationships.
- [Unicode LDML numbers](https://www.unicode.org/reports/tr35/tr35-78/tr35-numbers.html) for keeping currency identity separate from locale and making formatting/rounding behavior explicit.
- [IFRS IAS 2 overview](https://www.ifrs.org/issued-standards/list-of-standards/ias-2-inventories/) for the useful distinction among purchase, conversion/direct-labor, and production-overhead cost categories; PartProbe does not thereby claim accounting compliance.
- [NIST SP 800-53 Rev. 5 AU-3](https://doi.org/10.6028/NIST.SP.800-53r5) for audit-event content as a design reference; PartProbe does not thereby claim NIST compliance.
