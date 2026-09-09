use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{CurrencyCode, DomainError, EffectiveDate, Money, SourceRef};

macro_rules! bounded_id {
    ($(#[$attribute:meta])* $name:ident, $field:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                validate_text(&value, $field, 256)?;
                Ok(Self(value))
            }

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
                Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

bounded_id!(
    /// Stable identity of a shop resource-library draft.
    ShopResourceLibraryId,
    "shop resource-library ID"
);
bounded_id!(
    /// Stable identity of a material definition.
    MaterialDefinitionId,
    "material-definition ID"
);
bounded_id!(
    /// Stable identity of a commercial material offer.
    MaterialOfferId,
    "material-offer ID"
);
bounded_id!(
    /// Stable identity of a stock-allowance profile.
    StockAllowanceProfileId,
    "stock-allowance profile ID"
);
bounded_id!(
    /// Stable identity of a physical machine-capability profile.
    MachineProfileId,
    "machine-profile ID"
);
bounded_id!(
    /// Stable identity of a coarse runtime profile.
    RuntimeProfileId,
    "runtime-profile ID"
);

/// Positive immutable version shared by the bounded shop-resource records.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ShopResourceVersion(u32);

impl ShopResourceVersion {
    pub fn new(value: u32) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::InvalidValue {
                field: "shop resource version",
                reason: "must be greater than zero",
            });
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ShopResourceVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

macro_rules! decimal_measure {
    ($(#[$attribute:meta])* $name:ident, $field:literal, $positive:literal) => {
        $(#[$attribute])*
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(#[serde(with = "crate::decimal_serde")] Decimal);

        impl $name {
            pub fn new(value: Decimal) -> Result<Self, DomainError> {
                let invalid = if $positive {
                    value <= Decimal::ZERO
                } else {
                    value.is_sign_negative() && !value.is_zero()
                };
                if invalid {
                    return Err(DomainError::InvalidValue {
                        field: $field,
                        reason: if $positive { "must be greater than zero" } else { "must be non-negative" },
                    });
                }
                Ok(Self(value))
            }

            #[must_use]
            pub const fn value(self) -> Decimal {
                self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = crate::decimal_serde::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }
    };
}

decimal_measure!(
    /// Material density expressed in kilograms per cubic metre.
    DensityKilogramsPerCubicMeter,
    "material density",
    true
);
decimal_measure!(
    /// A stock-envelope allowance expressed in millimetres.
    StockAllowanceMillimeters,
    "stock allowance",
    false
);
decimal_measure!(
    /// A positive machine envelope dimension expressed in millimetres.
    MachineEnvelopeMillimeters,
    "machine envelope",
    true
);
decimal_measure!(
    /// Coarse volumetric removal rate expressed in cubic millimetres per minute.
    RemovalRateCubicMillimetersPerMinute,
    "coarse removal rate",
    true
);
decimal_measure!(
    /// A non-negative coarse duration expressed in minutes.
    RuntimeMinutes,
    "coarse runtime duration",
    false
);

/// Lifecycle evidence carried by a reusable library record.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryRecordState {
    Draft,
    Reviewed,
    Approved,
    Retired,
    Superseded,
}

/// Stock form interpreted by the first exact-STEP stock-proposal slice.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockForm {
    Rectangular,
    Round,
    Plate,
}

/// Broad process class used only for coarse routing/runtime proposals.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessClass {
    Milling,
    Turning,
    Sawing,
    Inspection,
}

/// Versioned physical material evidence, separate from any commercial price.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "MaterialDefinitionWire")]
pub struct MaterialDefinition {
    id: MaterialDefinitionId,
    version: ShopResourceVersion,
    family: String,
    grade: String,
    specification: Option<String>,
    condition: Option<String>,
    density_kg_per_m3: DensityKilogramsPerCubicMeter,
    source: SourceRef,
    state: LibraryRecordState,
}

#[derive(Deserialize)]
struct MaterialDefinitionWire {
    id: MaterialDefinitionId,
    version: ShopResourceVersion,
    family: String,
    grade: String,
    specification: Option<String>,
    condition: Option<String>,
    density_kg_per_m3: DensityKilogramsPerCubicMeter,
    source: SourceRef,
    state: LibraryRecordState,
}

impl TryFrom<MaterialDefinitionWire> for MaterialDefinition {
    type Error = DomainError;

    fn try_from(wire: MaterialDefinitionWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.id,
            wire.version,
            wire.family,
            wire.grade,
            wire.specification,
            wire.condition,
            wire.density_kg_per_m3,
            wire.source,
            wire.state,
        )
    }
}

impl MaterialDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MaterialDefinitionId,
        version: ShopResourceVersion,
        family: impl Into<String>,
        grade: impl Into<String>,
        specification: Option<String>,
        condition: Option<String>,
        density_kg_per_m3: DensityKilogramsPerCubicMeter,
        source: SourceRef,
        state: LibraryRecordState,
    ) -> Result<Self, DomainError> {
        let family = family.into();
        let grade = grade.into();
        validate_text(&family, "material family", 256)?;
        validate_text(&grade, "material grade", 256)?;
        validate_optional_text(specification.as_deref(), "material specification", 512)?;
        validate_optional_text(condition.as_deref(), "material condition", 256)?;
        Ok(Self {
            id,
            version,
            family,
            grade,
            specification,
            condition,
            density_kg_per_m3,
            source,
            state,
        })
    }

    #[must_use]
    pub const fn id(&self) -> &MaterialDefinitionId {
        &self.id
    }
    #[must_use]
    pub const fn version(&self) -> ShopResourceVersion {
        self.version
    }
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }
    #[must_use]
    pub fn grade(&self) -> &str {
        &self.grade
    }
    #[must_use]
    pub fn specification(&self) -> Option<&str> {
        self.specification.as_deref()
    }
    #[must_use]
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
    #[must_use]
    pub const fn density_kg_per_m3(&self) -> DensityKilogramsPerCubicMeter {
        self.density_kg_per_m3
    }
    #[must_use]
    pub const fn source(&self) -> &SourceRef {
        &self.source
    }
    #[must_use]
    pub const fn state(&self) -> LibraryRecordState {
        self.state
    }
}

/// Time-bounded commercial material price, separate from material identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "MaterialOfferWire")]
pub struct MaterialOffer {
    id: MaterialOfferId,
    version: ShopResourceVersion,
    material_id: MaterialDefinitionId,
    material_version: ShopResourceVersion,
    supplier: String,
    price_per_kg: Money,
    effective_from: EffectiveDate,
    source: SourceRef,
    state: LibraryRecordState,
}

#[derive(Deserialize)]
struct MaterialOfferWire {
    id: MaterialOfferId,
    version: ShopResourceVersion,
    material_id: MaterialDefinitionId,
    material_version: ShopResourceVersion,
    supplier: String,
    price_per_kg: Money,
    effective_from: EffectiveDate,
    source: SourceRef,
    state: LibraryRecordState,
}

impl TryFrom<MaterialOfferWire> for MaterialOffer {
    type Error = DomainError;

    fn try_from(wire: MaterialOfferWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.id,
            wire.version,
            wire.material_id,
            wire.material_version,
            wire.supplier,
            wire.price_per_kg,
            wire.effective_from,
            wire.source,
            wire.state,
        )
    }
}

impl MaterialOffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MaterialOfferId,
        version: ShopResourceVersion,
        material_id: MaterialDefinitionId,
        material_version: ShopResourceVersion,
        supplier: impl Into<String>,
        price_per_kg: Money,
        effective_from: EffectiveDate,
        source: SourceRef,
        state: LibraryRecordState,
    ) -> Result<Self, DomainError> {
        let supplier = supplier.into();
        validate_text(&supplier, "material supplier", 256)?;
        if price_per_kg.amount().is_sign_negative() && !price_per_kg.amount().is_zero() {
            return Err(DomainError::InvalidValue {
                field: "material price per kilogram",
                reason: "must be non-negative",
            });
        }
        Ok(Self {
            id,
            version,
            material_id,
            material_version,
            supplier,
            price_per_kg,
            effective_from,
            source,
            state,
        })
    }

    #[must_use]
    pub const fn id(&self) -> &MaterialOfferId {
        &self.id
    }
    #[must_use]
    pub const fn version(&self) -> ShopResourceVersion {
        self.version
    }
    #[must_use]
    pub const fn material_id(&self) -> &MaterialDefinitionId {
        &self.material_id
    }
    #[must_use]
    pub const fn material_version(&self) -> ShopResourceVersion {
        self.material_version
    }
    #[must_use]
    pub fn supplier(&self) -> &str {
        &self.supplier
    }
    #[must_use]
    pub const fn price_per_kg(&self) -> &Money {
        &self.price_per_kg
    }
    #[must_use]
    pub const fn effective_from(&self) -> &EffectiveDate {
        &self.effective_from
    }
    #[must_use]
    pub const fn source(&self) -> &SourceRef {
        &self.source
    }
    #[must_use]
    pub const fn state(&self) -> LibraryRecordState {
        self.state
    }
}

/// Versioned stock-envelope allowance policy. It is not a selected stock item.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StockAllowanceProfile {
    id: StockAllowanceProfileId,
    version: ShopResourceVersion,
    stock_form: StockForm,
    x_allowance_mm: StockAllowanceMillimeters,
    y_allowance_mm: StockAllowanceMillimeters,
    z_allowance_mm: StockAllowanceMillimeters,
    source: SourceRef,
    state: LibraryRecordState,
}

impl StockAllowanceProfile {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        id: StockAllowanceProfileId,
        version: ShopResourceVersion,
        stock_form: StockForm,
        x_allowance_mm: StockAllowanceMillimeters,
        y_allowance_mm: StockAllowanceMillimeters,
        z_allowance_mm: StockAllowanceMillimeters,
        source: SourceRef,
        state: LibraryRecordState,
    ) -> Self {
        Self {
            id,
            version,
            stock_form,
            x_allowance_mm,
            y_allowance_mm,
            z_allowance_mm,
            source,
            state,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &StockAllowanceProfileId {
        &self.id
    }
    #[must_use]
    pub const fn version(&self) -> ShopResourceVersion {
        self.version
    }
    #[must_use]
    pub const fn stock_form(&self) -> StockForm {
        self.stock_form
    }
    #[must_use]
    pub const fn x_allowance_mm(&self) -> StockAllowanceMillimeters {
        self.x_allowance_mm
    }
    #[must_use]
    pub const fn y_allowance_mm(&self) -> StockAllowanceMillimeters {
        self.y_allowance_mm
    }
    #[must_use]
    pub const fn z_allowance_mm(&self) -> StockAllowanceMillimeters {
        self.z_allowance_mm
    }
    #[must_use]
    pub const fn source(&self) -> &SourceRef {
        &self.source
    }
    #[must_use]
    pub const fn state(&self) -> LibraryRecordState {
        self.state
    }
}

/// Versioned physical machine capability, deliberately separate from financial rates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "MachineProfileWire")]
pub struct MachineProfile {
    id: MachineProfileId,
    version: ShopResourceVersion,
    name: String,
    process_class: ProcessClass,
    envelope_x_mm: MachineEnvelopeMillimeters,
    envelope_y_mm: MachineEnvelopeMillimeters,
    envelope_z_mm: MachineEnvelopeMillimeters,
    source: SourceRef,
    state: LibraryRecordState,
}

#[derive(Deserialize)]
struct MachineProfileWire {
    id: MachineProfileId,
    version: ShopResourceVersion,
    name: String,
    process_class: ProcessClass,
    envelope_x_mm: MachineEnvelopeMillimeters,
    envelope_y_mm: MachineEnvelopeMillimeters,
    envelope_z_mm: MachineEnvelopeMillimeters,
    source: SourceRef,
    state: LibraryRecordState,
}

impl TryFrom<MachineProfileWire> for MachineProfile {
    type Error = DomainError;

    fn try_from(wire: MachineProfileWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.id,
            wire.version,
            wire.name,
            wire.process_class,
            wire.envelope_x_mm,
            wire.envelope_y_mm,
            wire.envelope_z_mm,
            wire.source,
            wire.state,
        )
    }
}

impl MachineProfile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MachineProfileId,
        version: ShopResourceVersion,
        name: impl Into<String>,
        process_class: ProcessClass,
        envelope_x_mm: MachineEnvelopeMillimeters,
        envelope_y_mm: MachineEnvelopeMillimeters,
        envelope_z_mm: MachineEnvelopeMillimeters,
        source: SourceRef,
        state: LibraryRecordState,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        validate_text(&name, "machine name", 256)?;
        Ok(Self {
            id,
            version,
            name,
            process_class,
            envelope_x_mm,
            envelope_y_mm,
            envelope_z_mm,
            source,
            state,
        })
    }

    #[must_use]
    pub const fn id(&self) -> &MachineProfileId {
        &self.id
    }
    #[must_use]
    pub const fn version(&self) -> ShopResourceVersion {
        self.version
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn process_class(&self) -> ProcessClass {
        self.process_class
    }
    #[must_use]
    pub const fn envelope_x_mm(&self) -> MachineEnvelopeMillimeters {
        self.envelope_x_mm
    }
    #[must_use]
    pub const fn envelope_y_mm(&self) -> MachineEnvelopeMillimeters {
        self.envelope_y_mm
    }
    #[must_use]
    pub const fn envelope_z_mm(&self) -> MachineEnvelopeMillimeters {
        self.envelope_z_mm
    }
    #[must_use]
    pub const fn source(&self) -> &SourceRef {
        &self.source
    }
    #[must_use]
    pub const fn state(&self) -> LibraryRecordState {
        self.state
    }
}

/// Inputs for a future versioned coarse-volumetric runtime proposal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CoarseRuntimeProfile {
    id: RuntimeProfileId,
    version: ShopResourceVersion,
    machine_id: MachineProfileId,
    machine_version: ShopResourceVersion,
    material_id: MaterialDefinitionId,
    material_version: ShopResourceVersion,
    removal_rate_mm3_per_minute: RemovalRateCubicMillimetersPerMinute,
    setup_minutes: RuntimeMinutes,
    programming_minutes: RuntimeMinutes,
    load_unload_minutes: RuntimeMinutes,
    inspection_minutes: RuntimeMinutes,
    source: SourceRef,
    state: LibraryRecordState,
}

impl CoarseRuntimeProfile {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        id: RuntimeProfileId,
        version: ShopResourceVersion,
        machine_id: MachineProfileId,
        machine_version: ShopResourceVersion,
        material_id: MaterialDefinitionId,
        material_version: ShopResourceVersion,
        removal_rate_mm3_per_minute: RemovalRateCubicMillimetersPerMinute,
        setup_minutes: RuntimeMinutes,
        programming_minutes: RuntimeMinutes,
        load_unload_minutes: RuntimeMinutes,
        inspection_minutes: RuntimeMinutes,
        source: SourceRef,
        state: LibraryRecordState,
    ) -> Self {
        Self {
            id,
            version,
            machine_id,
            machine_version,
            material_id,
            material_version,
            removal_rate_mm3_per_minute,
            setup_minutes,
            programming_minutes,
            load_unload_minutes,
            inspection_minutes,
            source,
            state,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &RuntimeProfileId {
        &self.id
    }
    #[must_use]
    pub const fn version(&self) -> ShopResourceVersion {
        self.version
    }
    #[must_use]
    pub const fn machine_id(&self) -> &MachineProfileId {
        &self.machine_id
    }
    #[must_use]
    pub const fn machine_version(&self) -> ShopResourceVersion {
        self.machine_version
    }
    #[must_use]
    pub const fn material_id(&self) -> &MaterialDefinitionId {
        &self.material_id
    }
    #[must_use]
    pub const fn material_version(&self) -> ShopResourceVersion {
        self.material_version
    }
    #[must_use]
    pub const fn removal_rate_mm3_per_minute(&self) -> RemovalRateCubicMillimetersPerMinute {
        self.removal_rate_mm3_per_minute
    }
    #[must_use]
    pub const fn setup_minutes(&self) -> RuntimeMinutes {
        self.setup_minutes
    }
    #[must_use]
    pub const fn programming_minutes(&self) -> RuntimeMinutes {
        self.programming_minutes
    }
    #[must_use]
    pub const fn load_unload_minutes(&self) -> RuntimeMinutes {
        self.load_unload_minutes
    }
    #[must_use]
    pub const fn inspection_minutes(&self) -> RuntimeMinutes {
        self.inspection_minutes
    }
    #[must_use]
    pub const fn source(&self) -> &SourceRef {
        &self.source
    }
    #[must_use]
    pub const fn state(&self) -> LibraryRecordState {
        self.state
    }
}

/// One bounded, internally consistent shop-resource draft used by the next proposal slices.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ShopResourceLibraryWire")]
pub struct ShopResourceLibrary {
    id: ShopResourceLibraryId,
    version: ShopResourceVersion,
    currency: CurrencyCode,
    material: MaterialDefinition,
    material_offer: MaterialOffer,
    stock_allowance: StockAllowanceProfile,
    machine: MachineProfile,
    runtime: CoarseRuntimeProfile,
}

#[derive(Deserialize)]
struct ShopResourceLibraryWire {
    id: ShopResourceLibraryId,
    version: ShopResourceVersion,
    currency: CurrencyCode,
    material: MaterialDefinition,
    material_offer: MaterialOffer,
    stock_allowance: StockAllowanceProfile,
    machine: MachineProfile,
    runtime: CoarseRuntimeProfile,
}

impl TryFrom<ShopResourceLibraryWire> for ShopResourceLibrary {
    type Error = DomainError;

    fn try_from(wire: ShopResourceLibraryWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.id,
            wire.version,
            wire.currency,
            wire.material,
            wire.material_offer,
            wire.stock_allowance,
            wire.machine,
            wire.runtime,
        )
    }
}

impl ShopResourceLibrary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ShopResourceLibraryId,
        version: ShopResourceVersion,
        currency: CurrencyCode,
        material: MaterialDefinition,
        material_offer: MaterialOffer,
        stock_allowance: StockAllowanceProfile,
        machine: MachineProfile,
        runtime: CoarseRuntimeProfile,
    ) -> Result<Self, DomainError> {
        if material_offer.material_id() != material.id()
            || material_offer.material_version() != material.version()
            || runtime.material_id() != material.id()
            || runtime.material_version() != material.version()
            || runtime.machine_id() != machine.id()
            || runtime.machine_version() != machine.version()
        {
            return Err(DomainError::InvalidValue {
                field: "shop resource references",
                reason: "offer and runtime records must reference the bundled material and machine versions",
            });
        }
        if material_offer.price_per_kg().currency() != &currency {
            return Err(DomainError::CurrencyMismatch {
                left: currency.as_str().to_owned(),
                right: material_offer.price_per_kg().currency().as_str().to_owned(),
            });
        }
        Ok(Self {
            id,
            version,
            currency,
            material,
            material_offer,
            stock_allowance,
            machine,
            runtime,
        })
    }

    #[must_use]
    pub const fn id(&self) -> &ShopResourceLibraryId {
        &self.id
    }
    #[must_use]
    pub const fn version(&self) -> ShopResourceVersion {
        self.version
    }
    #[must_use]
    pub const fn currency(&self) -> &CurrencyCode {
        &self.currency
    }
    #[must_use]
    pub const fn material(&self) -> &MaterialDefinition {
        &self.material
    }
    #[must_use]
    pub const fn material_offer(&self) -> &MaterialOffer {
        &self.material_offer
    }
    #[must_use]
    pub const fn stock_allowance(&self) -> &StockAllowanceProfile {
        &self.stock_allowance
    }
    #[must_use]
    pub const fn machine(&self) -> &MachineProfile {
        &self.machine
    }
    #[must_use]
    pub const fn runtime(&self) -> &CoarseRuntimeProfile {
        &self.runtime
    }
}

fn validate_text(value: &str, field: &'static str, maximum: usize) -> Result<(), DomainError> {
    if value.trim().is_empty() || value.len() > maximum || value.contains('\0') {
        return Err(DomainError::InvalidValue {
            field,
            reason: "must be nonempty, bounded, and contain no null byte",
        });
    }
    Ok(())
}

fn validate_optional_text(
    value: Option<&str>,
    field: &'static str,
    maximum: usize,
) -> Result<(), DomainError> {
    value.map_or(Ok(()), |value| validate_text(value, field, maximum))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{RecordedAt, SourceKind};

    fn decimal(value: &str) -> Decimal {
        Decimal::from_str(value).unwrap()
    }

    fn source(id: &str) -> SourceRef {
        SourceRef::new(
            SourceKind::Manual,
            id,
            None,
            Some(RecordedAt::new("2026-09-09T12:00:00Z").unwrap()),
        )
        .unwrap()
    }

    fn library(currency: &str) -> Result<ShopResourceLibrary, DomainError> {
        let version = ShopResourceVersion::new(1)?;
        let material_id = MaterialDefinitionId::new("al-6061-t6")?;
        let machine_id = MachineProfileId::new("vmc-1")?;
        let material = MaterialDefinition::new(
            material_id.clone(),
            version,
            "Aluminum",
            "6061",
            Some("ASTM B221".into()),
            Some("T6".into()),
            DensityKilogramsPerCubicMeter::new(decimal("2700"))?,
            source("material-handbook"),
            LibraryRecordState::Draft,
        )?;
        let offer = MaterialOffer::new(
            MaterialOfferId::new("al-6061-offer")?,
            version,
            material_id.clone(),
            version,
            "test supplier",
            Money::new(decimal("8.50"), CurrencyCode::new(currency)?),
            EffectiveDate::new("2026-09-09")?,
            source("supplier-test-quote"),
            LibraryRecordState::Draft,
        )?;
        let stock = StockAllowanceProfile::new(
            StockAllowanceProfileId::new("rectangular-default")?,
            version,
            StockForm::Rectangular,
            StockAllowanceMillimeters::new(decimal("3"))?,
            StockAllowanceMillimeters::new(decimal("3"))?,
            StockAllowanceMillimeters::new(decimal("2"))?,
            source("shop-review"),
            LibraryRecordState::Draft,
        );
        let machine = MachineProfile::new(
            machine_id.clone(),
            version,
            "Test VMC",
            ProcessClass::Milling,
            MachineEnvelopeMillimeters::new(decimal("762"))?,
            MachineEnvelopeMillimeters::new(decimal("508"))?,
            MachineEnvelopeMillimeters::new(decimal("508"))?,
            source("machine-manual"),
            LibraryRecordState::Draft,
        )?;
        let runtime = CoarseRuntimeProfile::new(
            RuntimeProfileId::new("vmc-al-coarse")?,
            version,
            machine_id,
            version,
            material_id,
            version,
            RemovalRateCubicMillimetersPerMinute::new(decimal("16000"))?,
            RuntimeMinutes::new(decimal("60"))?,
            RuntimeMinutes::new(decimal("45"))?,
            RuntimeMinutes::new(decimal("3"))?,
            RuntimeMinutes::new(decimal("15"))?,
            source("shop-estimator-review"),
            LibraryRecordState::Draft,
        );
        ShopResourceLibrary::new(
            ShopResourceLibraryId::new("starter-resources")?,
            version,
            CurrencyCode::new("USD")?,
            material,
            offer,
            stock,
            machine,
            runtime,
        )
    }

    #[test]
    fn resource_library_preserves_typed_records_and_round_trips() {
        let value = library("USD").unwrap();
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(
            serde_json::from_str::<ShopResourceLibrary>(&json).unwrap(),
            value
        );
    }

    #[test]
    fn resource_library_rejects_offer_currency_mismatch() {
        assert!(matches!(
            library("EUR"),
            Err(DomainError::CurrencyMismatch { .. })
        ));
    }

    #[test]
    fn resource_measures_reject_invalid_signs() {
        assert!(DensityKilogramsPerCubicMeter::new(Decimal::ZERO).is_err());
        assert!(StockAllowanceMillimeters::new(decimal("-0.1")).is_err());
        assert!(RuntimeMinutes::new(Decimal::ZERO).is_ok());
    }

    #[test]
    fn deserialization_revalidates_text_and_cross_record_references() {
        let value = library("USD").unwrap();
        let mut invalid_text = serde_json::to_value(&value).unwrap();
        invalid_text["material"]["family"] = serde_json::json!("");
        assert!(serde_json::from_value::<ShopResourceLibrary>(invalid_text).is_err());

        let mut invalid_reference = serde_json::to_value(&value).unwrap();
        invalid_reference["runtime"]["machine_id"] = serde_json::json!("another-machine");
        assert!(serde_json::from_value::<ShopResourceLibrary>(invalid_reference).is_err());

        let mut invalid_price = serde_json::to_value(&value).unwrap();
        invalid_price["material_offer"]["price_per_kg"]["amount"] = serde_json::json!("-1");
        assert!(serde_json::from_value::<ShopResourceLibrary>(invalid_price).is_err());
    }
}
