use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    ActorId, CurrencyCode, DomainError, PricingPolicy, RateCard, RecordedAt, ShopResourceLibrary,
};

/// Stable identity of one shop-settings profile.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ShopProfileId(String);

impl ShopProfileId {
    /// Validates and creates a profile identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 256 || value.contains('\0') {
            return Err(DomainError::InvalidValue {
                field: "shop profile ID",
                reason: "must be nonempty, bounded to 256 bytes, and contain no null byte",
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

impl<'de> Deserialize<'de> for ShopProfileId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Positive immutable revision of a shop-settings draft.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ShopSettingsRevision(u32);

impl ShopSettingsRevision {
    /// Validates and creates a revision.
    pub fn new(value: u32) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::InvalidValue {
                field: "shop settings revision",
                reason: "must be greater than zero",
            });
        }
        Ok(Self(value))
    }

    /// Returns the numeric revision.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ShopSettingsRevision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Immutable, replayable draft of reusable shop calculation settings.
///
/// Persistence alone does not make this draft calculation authority. An application workflow
/// must separately review and activate its governed rates, pricing, and resource records.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ShopSettingsDraft {
    profile_id: ShopProfileId,
    revision: ShopSettingsRevision,
    currency: CurrencyCode,
    rate_card: Option<RateCard>,
    pricing_policy: Option<PricingPolicy>,
    resource_library: Option<ShopResourceLibrary>,
    changed_by: ActorId,
    changed_at: RecordedAt,
    change_reason: String,
}

#[derive(Deserialize)]
struct ShopSettingsDraftWire {
    profile_id: ShopProfileId,
    revision: ShopSettingsRevision,
    currency: CurrencyCode,
    rate_card: Option<RateCard>,
    pricing_policy: Option<PricingPolicy>,
    #[serde(default)]
    resource_library: Option<ShopResourceLibrary>,
    changed_by: ActorId,
    changed_at: RecordedAt,
    change_reason: String,
}

impl<'de> Deserialize<'de> for ShopSettingsDraft {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ShopSettingsDraftWire::deserialize(deserializer)?;
        Self::new_with_resources(
            wire.profile_id,
            wire.revision,
            wire.currency,
            wire.rate_card,
            wire.pricing_policy,
            wire.resource_library,
            wire.changed_by,
            wire.changed_at,
            wire.change_reason,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl ShopSettingsDraft {
    /// Creates a validated draft without introducing numeric defaults.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        profile_id: ShopProfileId,
        revision: ShopSettingsRevision,
        currency: CurrencyCode,
        rate_card: Option<RateCard>,
        pricing_policy: Option<PricingPolicy>,
        changed_by: ActorId,
        changed_at: RecordedAt,
        change_reason: impl Into<String>,
    ) -> Result<Self, DomainError> {
        Self::new_with_resources(
            profile_id,
            revision,
            currency,
            rate_card,
            pricing_policy,
            None,
            changed_by,
            changed_at,
            change_reason,
        )
    }

    /// Creates a validated draft with an optional, separately versioned resource library.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_resources(
        profile_id: ShopProfileId,
        revision: ShopSettingsRevision,
        currency: CurrencyCode,
        rate_card: Option<RateCard>,
        pricing_policy: Option<PricingPolicy>,
        resource_library: Option<ShopResourceLibrary>,
        changed_by: ActorId,
        changed_at: RecordedAt,
        change_reason: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let change_reason = change_reason.into();
        if change_reason.trim().is_empty()
            || change_reason.len() > 2_048
            || change_reason.contains('\0')
        {
            return Err(DomainError::InvalidValue {
                field: "shop settings change reason",
                reason: "must be nonempty, bounded to 2048 bytes, and contain no null byte",
            });
        }
        if let Some(card) = rate_card.as_ref() {
            ensure_currency(&currency, card.currency())?;
        }
        if let Some(policy) = pricing_policy.as_ref() {
            ensure_currency(&currency, policy.currency())?;
        }
        if let Some(resources) = resource_library.as_ref() {
            ensure_currency(&currency, resources.currency())?;
        }
        Ok(Self {
            profile_id,
            revision,
            currency,
            rate_card,
            pricing_policy,
            resource_library,
            changed_by,
            changed_at,
            change_reason,
        })
    }

    #[must_use]
    pub const fn profile_id(&self) -> &ShopProfileId {
        &self.profile_id
    }

    #[must_use]
    pub const fn revision(&self) -> ShopSettingsRevision {
        self.revision
    }

    #[must_use]
    pub const fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    #[must_use]
    pub const fn rate_card(&self) -> Option<&RateCard> {
        self.rate_card.as_ref()
    }

    #[must_use]
    pub const fn pricing_policy(&self) -> Option<&PricingPolicy> {
        self.pricing_policy.as_ref()
    }

    #[must_use]
    pub const fn resource_library(&self) -> Option<&ShopResourceLibrary> {
        self.resource_library.as_ref()
    }

    #[must_use]
    pub const fn changed_by(&self) -> &ActorId {
        &self.changed_by
    }

    #[must_use]
    pub const fn changed_at(&self) -> &RecordedAt {
        &self.changed_at
    }

    #[must_use]
    pub fn change_reason(&self) -> &str {
        &self.change_reason
    }
}

fn ensure_currency(expected: &CurrencyCode, actual: &CurrencyCode) -> Result<(), DomainError> {
    if expected == actual {
        Ok(())
    } else {
        Err(DomainError::CurrencyMismatch {
            left: expected.as_str().to_owned(),
            right: actual.as_str().to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RateCardId, RateVersion};

    fn draft(card_currency: &str) -> Result<ShopSettingsDraft, DomainError> {
        let currency = CurrencyCode::new("USD")?;
        ShopSettingsDraft::new(
            ShopProfileId::new("shop-1")?,
            ShopSettingsRevision::new(1)?,
            currency,
            Some(RateCard::empty(
                RateCardId::new("rates-1")?,
                RateVersion::new(1)?,
                CurrencyCode::new(card_currency)?,
            )),
            None,
            ActorId::new("operator-1")?,
            RecordedAt::new("2026-09-04T12:00:00Z")?,
            "initial draft",
        )
    }

    #[test]
    fn draft_preserves_missing_policy_without_defaulting() {
        let value = draft("USD").expect("valid draft");
        assert!(value.pricing_policy().is_none());
        assert_eq!(value.rate_card().expect("rate card").entries(), &[]);
    }

    #[test]
    fn draft_rejects_currency_mismatch() {
        assert!(matches!(
            draft("EUR"),
            Err(DomainError::CurrencyMismatch { .. })
        ));
    }

    #[test]
    fn deserialization_revalidates_the_revision() {
        let json = serde_json::to_string(&draft("USD").expect("valid draft")).expect("serialize");
        let invalid = json.replace("\"revision\":1", "\"revision\":0");
        assert!(serde_json::from_str::<ShopSettingsDraft>(&invalid).is_err());
    }

    #[test]
    fn prior_payload_without_resource_library_remains_readable() {
        let json = serde_json::to_string(&draft("USD").expect("valid draft")).expect("serialize");
        let prior = json.replace(",\"resource_library\":null", "");
        let restored: ShopSettingsDraft = serde_json::from_str(&prior).expect("read prior payload");
        assert!(restored.resource_library().is_none());
    }
}
