use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{CurrencyCode, DomainError, Money, RecordedAt, SourceRef};

macro_rules! non_empty_id {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validates and creates the identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: "must not be empty",
                    });
                }
                Ok(Self(value))
            }

            /// Returns the validated identifier.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

non_empty_id!(
    /// Stable identity of a reusable rate entry.
    RateId,
    "rate ID"
);
non_empty_id!(
    /// Stable identity of a versioned rate card.
    RateCardId,
    "rate-card ID"
);

/// Positive version number for rate cards and entries.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RateVersion(u32);

impl RateVersion {
    /// Validates and creates a rate version.
    pub fn new(value: u32) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::InvalidValue {
                field: "rate version",
                reason: "must be greater than zero",
            });
        }
        Ok(Self(value))
    }

    /// Returns the numeric version.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for RateVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Calendar date used for effective-period comparison.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct EffectiveDate(String);

impl EffectiveDate {
    /// Validates an ISO-style Gregorian `YYYY-MM-DD` date.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if !is_valid_date(&value) {
            return Err(DomainError::InvalidValue {
                field: "effective date",
                reason: "must be a valid Gregorian date in YYYY-MM-DD form",
            });
        }
        Ok(Self(value))
    }

    /// Returns the normalized date text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for EffectiveDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Cost category to which a rate or commercial charge contributes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostCategory {
    /// Purchased material and stock.
    Material,
    /// Programming effort.
    Programming,
    /// Setup labor.
    SetupLabor,
    /// Recurring run labor.
    RunLabor,
    /// Machine or workcenter occupancy.
    Machine,
    /// Explicit burden component.
    Burden,
    /// Cutting tools and replacement allowance.
    Tooling,
    /// Consumable supplies.
    Consumables,
    /// Fixture design, build, or use.
    Fixture,
    /// Inspection and quality effort.
    QualityInspection,
    /// Outside processing.
    OutsideProcess,
    /// Freight and delivery charges.
    Freight,
    /// Nonrecurring engineering.
    NonRecurringEngineering,
    /// Administrative effort.
    Administration,
    /// Explicit overhead allocation.
    Overhead,
}

/// Unit basis carried by a rate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateBasis {
    /// Charge per hour.
    PerHour,
    /// Charge per minute.
    PerMinute,
    /// Charge per produced item.
    PerItem,
    /// Charge per lot.
    PerLot,
    /// Charge per setup occurrence.
    PerSetup,
    /// Charge per kilogram.
    PerKilogram,
    /// Charge per meter.
    PerMeter,
    /// One flat charge.
    Flat,
}

/// Whether a rate is an atomic charge or an intentionally bundled charge.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateComposition {
    /// One independently extended cost component.
    Component,
    /// One bundled rate that must not be combined with component rates.
    Composite,
}

/// Kind of reusable applicability scope.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateScopeKind {
    /// Organization-wide fallback.
    Organization,
    /// Physical or logical site.
    Site,
    /// Financial cost center.
    CostCenter,
    /// Workcenter class or instance.
    Workcenter,
    /// Specific machine.
    Machine,
    /// Operation class.
    Operation,
    /// Labor classification.
    LaborClass,
    /// Material class or definition.
    Material,
    /// Outside vendor.
    Vendor,
}

/// One explicit applicability scope for a rate.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RateScope {
    kind: RateScopeKind,
    reference_id: Option<String>,
}

#[derive(Deserialize)]
struct RateScopeWire {
    kind: RateScopeKind,
    reference_id: Option<String>,
}

impl<'de> Deserialize<'de> for RateScope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RateScopeWire::deserialize(deserializer)?;
        Self::new(wire.kind, wire.reference_id).map_err(serde::de::Error::custom)
    }
}

impl RateScope {
    /// Creates an organization-wide scope.
    #[must_use]
    pub const fn organization() -> Self {
        Self {
            kind: RateScopeKind::Organization,
            reference_id: None,
        }
    }

    /// Validates and creates a scope.
    pub fn new(kind: RateScopeKind, reference_id: Option<String>) -> Result<Self, DomainError> {
        match (kind, reference_id.as_deref()) {
            (RateScopeKind::Organization, None) => {}
            (RateScopeKind::Organization, Some(_)) => {
                return Err(DomainError::InvalidValue {
                    field: "rate scope",
                    reason: "organization scope must not carry a reference ID",
                });
            }
            (_, Some(reference_id)) if !reference_id.trim().is_empty() => {}
            _ => {
                return Err(DomainError::InvalidValue {
                    field: "rate scope",
                    reason: "non-organization scope requires a reference ID",
                });
            }
        }
        Ok(Self { kind, reference_id })
    }

    /// Returns the scope kind.
    #[must_use]
    pub const fn kind(&self) -> RateScopeKind {
        self.kind
    }

    /// Returns the optional referenced object ID.
    #[must_use]
    pub fn reference_id(&self) -> Option<&str> {
        self.reference_id.as_deref()
    }
}

/// Governance state of a reusable rate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateApprovalState {
    /// Editable proposal.
    Draft,
    /// Reviewed but not authorized for calculations.
    Reviewed,
    /// Authorized for effective-date resolution.
    Approved,
    /// Retained for history but unavailable for new selection.
    Retired,
    /// Replaced by a later immutable version and retained for replay.
    Superseded,
}

/// Actor, time, and reason for a governed rate event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RateEvent {
    actor: String,
    recorded_at: RecordedAt,
    reason: String,
}

#[derive(Deserialize)]
struct RateEventWire {
    actor: String,
    recorded_at: RecordedAt,
    reason: String,
}

impl<'de> Deserialize<'de> for RateEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RateEventWire::deserialize(deserializer)?;
        Self::new(wire.actor, wire.recorded_at, wire.reason).map_err(serde::de::Error::custom)
    }
}

impl RateEvent {
    /// Validates and creates a governed event.
    pub fn new(
        actor: impl Into<String>,
        recorded_at: RecordedAt,
        reason: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let actor = actor.into();
        let reason = reason.into();
        if actor.trim().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "rate event actor",
                reason: "must not be empty",
            });
        }
        if reason.trim().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "rate event reason",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            actor,
            recorded_at,
            reason,
        })
    }

    /// Returns the actor.
    #[must_use]
    pub fn actor(&self) -> &str {
        &self.actor
    }

    /// Returns the recorded time.
    #[must_use]
    pub const fn recorded_at(&self) -> &RecordedAt {
        &self.recorded_at
    }

    /// Returns the reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Entry and optional review/approval history for one rate version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RateGovernance {
    state: RateApprovalState,
    entered: RateEvent,
    prior_decisions: Vec<RateEvent>,
    decision: Option<RateEvent>,
}

#[derive(Deserialize)]
struct RateGovernanceWire {
    state: RateApprovalState,
    entered: RateEvent,
    #[serde(default)]
    prior_decisions: Vec<RateEvent>,
    decision: Option<RateEvent>,
}

impl<'de> Deserialize<'de> for RateGovernance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RateGovernanceWire::deserialize(deserializer)?;
        Self::with_prior_decisions(
            wire.state,
            wire.entered,
            wire.prior_decisions,
            wire.decision,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl RateGovernance {
    /// Validates state-specific governance evidence.
    pub fn new(
        state: RateApprovalState,
        entered: RateEvent,
        decision: Option<RateEvent>,
    ) -> Result<Self, DomainError> {
        Self::with_prior_decisions(state, entered, Vec::new(), decision)
    }

    /// Validates governance while preserving decisions from earlier lifecycle states.
    pub fn with_prior_decisions(
        state: RateApprovalState,
        entered: RateEvent,
        prior_decisions: Vec<RateEvent>,
        decision: Option<RateEvent>,
    ) -> Result<Self, DomainError> {
        let decision_required = state != RateApprovalState::Draft;
        if decision_required != decision.is_some() {
            return Err(DomainError::InvalidValue {
                field: "rate governance",
                reason: "non-draft states require a decision and draft state must not carry one",
            });
        }
        Ok(Self {
            state,
            entered,
            prior_decisions,
            decision,
        })
    }

    /// Returns the current state.
    #[must_use]
    pub const fn state(&self) -> RateApprovalState {
        self.state
    }

    /// Returns the entry event.
    #[must_use]
    pub const fn entered(&self) -> &RateEvent {
        &self.entered
    }

    /// Returns decisions retained from earlier lifecycle states.
    #[must_use]
    pub fn prior_decisions(&self) -> &[RateEvent] {
        &self.prior_decisions
    }

    /// Returns the optional review, approval, or retirement decision.
    #[must_use]
    pub const fn decision(&self) -> Option<&RateEvent> {
        self.decision.as_ref()
    }
}

/// Immutable reusable rate entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RateEntry {
    id: RateId,
    version: RateVersion,
    category: CostCategory,
    composition: RateComposition,
    scope: RateScope,
    owner: String,
    amount: Money,
    basis: RateBasis,
    effective_from: EffectiveDate,
    effective_through: Option<EffectiveDate>,
    governance: RateGovernance,
    source: SourceRef,
}

#[derive(Deserialize)]
struct RateEntryWire {
    id: RateId,
    version: RateVersion,
    category: CostCategory,
    composition: RateComposition,
    scope: RateScope,
    owner: String,
    amount: Money,
    basis: RateBasis,
    effective_from: EffectiveDate,
    effective_through: Option<EffectiveDate>,
    governance: RateGovernance,
    source: SourceRef,
}

impl<'de> Deserialize<'de> for RateEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RateEntryWire::deserialize(deserializer)?;
        Self::new(
            wire.id,
            wire.version,
            wire.category,
            wire.composition,
            wire.scope,
            wire.owner,
            wire.amount,
            wire.basis,
            wire.effective_from,
            wire.effective_through,
            wire.governance,
            wire.source,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl RateEntry {
    /// Validates and creates an immutable rate entry.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RateId,
        version: RateVersion,
        category: CostCategory,
        composition: RateComposition,
        scope: RateScope,
        owner: impl Into<String>,
        amount: Money,
        basis: RateBasis,
        effective_from: EffectiveDate,
        effective_through: Option<EffectiveDate>,
        governance: RateGovernance,
        source: SourceRef,
    ) -> Result<Self, DomainError> {
        let owner = owner.into();
        if owner.trim().is_empty() {
            return Err(DomainError::InvalidValue {
                field: "rate owner",
                reason: "must not be empty",
            });
        }
        if amount.amount().is_sign_negative() {
            return Err(DomainError::InvalidValue {
                field: "rate amount",
                reason: "must be nonnegative",
            });
        }
        if effective_through
            .as_ref()
            .is_some_and(|through| through < &effective_from)
        {
            return Err(DomainError::InvalidValue {
                field: "rate effective period",
                reason: "effective-through date must not precede effective-from date",
            });
        }
        Ok(Self {
            id,
            version,
            category,
            composition,
            scope,
            owner,
            amount,
            basis,
            effective_from,
            effective_through,
            governance,
            source,
        })
    }

    /// Returns whether this approved entry is effective on the date.
    #[must_use]
    pub fn is_approved_and_effective(&self, date: &EffectiveDate) -> bool {
        self.governance.state == RateApprovalState::Approved
            && &self.effective_from <= date
            && self
                .effective_through
                .as_ref()
                .is_none_or(|through| date <= through)
    }

    /// Returns the stable ID.
    #[must_use]
    pub const fn id(&self) -> &RateId {
        &self.id
    }

    /// Returns the entry version.
    #[must_use]
    pub const fn version(&self) -> RateVersion {
        self.version
    }

    /// Returns the cost category.
    #[must_use]
    pub const fn category(&self) -> CostCategory {
        self.category
    }

    /// Returns whether this is an atomic component or a bundled rate.
    #[must_use]
    pub const fn composition(&self) -> RateComposition {
        self.composition
    }

    /// Returns the applicability scope.
    #[must_use]
    pub const fn scope(&self) -> &RateScope {
        &self.scope
    }

    /// Returns the accountable owner of this rate version.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Returns the exact amount.
    #[must_use]
    pub const fn amount(&self) -> &Money {
        &self.amount
    }

    /// Returns the rate basis.
    #[must_use]
    pub const fn basis(&self) -> RateBasis {
        self.basis
    }

    /// Returns the effective-from date.
    #[must_use]
    pub const fn effective_from(&self) -> &EffectiveDate {
        &self.effective_from
    }

    /// Returns the optional effective-through date.
    #[must_use]
    pub const fn effective_through(&self) -> Option<&EffectiveDate> {
        self.effective_through.as_ref()
    }

    /// Returns the governance record.
    #[must_use]
    pub const fn governance(&self) -> &RateGovernance {
        &self.governance
    }

    /// Returns the source reference.
    #[must_use]
    pub const fn source(&self) -> &SourceRef {
        &self.source
    }
}

/// Immutable snapshot of reusable rates in one currency.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RateCard {
    id: RateCardId,
    version: RateVersion,
    currency: CurrencyCode,
    entries: Vec<RateEntry>,
}

#[derive(Deserialize)]
struct RateCardWire {
    id: RateCardId,
    version: RateVersion,
    currency: CurrencyCode,
    entries: Vec<RateEntry>,
}

impl<'de> Deserialize<'de> for RateCard {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RateCardWire::deserialize(deserializer)?;
        Self::new(wire.id, wire.version, wire.currency, wire.entries)
            .map_err(serde::de::Error::custom)
    }
}

impl RateCard {
    /// Creates an empty card; production initialization supplies no numeric defaults.
    #[must_use]
    pub const fn empty(id: RateCardId, version: RateVersion, currency: CurrencyCode) -> Self {
        Self {
            id,
            version,
            currency,
            entries: Vec::new(),
        }
    }

    /// Validates and creates a rate-card snapshot.
    pub fn new(
        id: RateCardId,
        version: RateVersion,
        currency: CurrencyCode,
        mut entries: Vec<RateEntry>,
    ) -> Result<Self, DomainError> {
        let mut ids = BTreeSet::new();
        for entry in &entries {
            if entry.amount.currency() != &currency {
                return Err(DomainError::CurrencyMismatch {
                    left: currency.as_str().to_owned(),
                    right: entry.amount.currency().as_str().to_owned(),
                });
            }
            if !ids.insert(entry.id.clone()) {
                return Err(DomainError::InvalidValue {
                    field: "rate-card entry",
                    reason: "stable rate IDs must be unique within a card version",
                });
            }
        }
        entries.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(Self {
            id,
            version,
            currency,
            entries,
        })
    }

    /// Returns the card ID.
    #[must_use]
    pub const fn id(&self) -> &RateCardId {
        &self.id
    }

    /// Returns the card version.
    #[must_use]
    pub const fn version(&self) -> RateVersion {
        self.version
    }

    /// Returns the card currency.
    #[must_use]
    pub const fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    /// Returns immutable entries.
    #[must_use]
    pub fn entries(&self) -> &[RateEntry] {
        &self.entries
    }
}

fn is_valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let Some(year) = parse_digits(&bytes[0..4]).filter(|year| *year > 0) else {
        return false;
    };
    let Some(month) = parse_digits(&bytes[5..7]) else {
        return false;
    };
    let Some(day) = parse_digits(&bytes[8..10]) else {
        return false;
    };
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };
    day >= 1 && day <= days
}

fn parse_digits(bytes: &[u8]) -> Option<u32> {
    bytes.iter().try_fold(0_u32, |value, byte| {
        byte.is_ascii_digit()
            .then(|| value * 10 + u32::from(byte - b'0'))
    })
}

fn is_leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
