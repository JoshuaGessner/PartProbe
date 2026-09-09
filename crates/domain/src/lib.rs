//! UI-independent domain primitives for PartProbe.

pub mod access;
#[doc(hidden)]
pub mod decimal_serde;
pub mod error;
pub mod money;
pub mod pricing;
pub mod provenance;
pub mod rates;
pub mod shop_resources;
pub mod shop_settings;
pub mod units;
pub mod value_state;

pub use access::{
    ActorId, AssetRootId, DataClassificationId, ProjectId, RecordId, RecordStateId, RecordVersionId,
};
pub use error::DomainError;
pub use money::{CurrencyCode, Money};
pub use pricing::{
    PricingMethod, PricingPolicy, PricingPolicyId, RoundedMoney, RoundingBoundary, RoundingMode,
    RoundingPolicy, RoundingPolicyId,
};
pub use provenance::{
    RecordedAt, RuleId, RuleRef, RuleVersion, SchemaVersion, SourceKind, SourceRef,
};
pub use rates::{
    CostCategory, EffectiveDate, RateApprovalState, RateBasis, RateCard, RateCardId,
    RateComposition, RateEntry, RateEvent, RateGovernance, RateId, RateScope, RateScopeKind,
    RateVersion,
};
pub use shop_resources::{
    CoarseRuntimeProfile, DensityKilogramsPerCubicMeter, LibraryRecordState,
    MAX_SHOP_RESOURCE_RECORDS_PER_KIND, MachineEnvelopeMillimeters, MachineProfile,
    MachineProfileId, MaterialDefinition, MaterialDefinitionId, MaterialOffer, MaterialOfferId,
    ProcessClass, RemovalRateCubicMillimetersPerMinute, ResourceSelection, ResourceSelectionId,
    ResourceSelectionState, RuntimeMinutes, RuntimeProfileId, SHOP_RESOURCE_CATALOG_SCHEMA,
    ShopResourceCatalog, ShopResourceCatalogId, ShopResourceLibrary, ShopResourceLibraryId,
    ShopResourceVersion, StockAllowanceMillimeters, StockAllowanceProfile, StockAllowanceProfileId,
    StockForm,
};
pub use shop_settings::{ShopProfileId, ShopSettingsDraft, ShopSettingsRevision};
pub use units::{
    DensityKilogramsPerCubicMillimeter, ItemQuantity, MassKilograms, VolumeCubicMillimeters,
};
pub use value_state::ValueState;
