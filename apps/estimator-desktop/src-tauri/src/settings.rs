use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use partprobe_application::{
    ShopSettingsApplication, ShopSettingsLoadState, ShopSettingsStoreError,
};
use partprobe_desktop_contract::{
    DeveloperPricingInputFields, DeveloperRateInputFields, HostCommandError,
    SaveShopSettingsRequest, ShopResourceInputFields, ShopSettingsSnapshot, ShopSettingsState,
};
use partprobe_domain::{
    ActorId, CoarseRuntimeProfile, CostCategory, CurrencyCode, DensityKilogramsPerCubicMeter,
    EffectiveDate, LibraryRecordState, MachineEnvelopeMillimeters, MachineProfile,
    MachineProfileId, MaterialDefinition, MaterialDefinitionId, MaterialOffer, MaterialOfferId,
    Money, PricingMethod, ProcessClass, RateCard, RecordedAt, RemovalRateCubicMillimetersPerMinute,
    RuntimeMinutes, RuntimeProfileId, ShopProfileId, ShopResourceLibrary, ShopResourceLibraryId,
    ShopResourceVersion, ShopSettingsDraft, ShopSettingsRevision, SourceKind, SourceRef,
    StockAllowanceMillimeters, StockAllowanceProfile, StockAllowanceProfileId, StockForm,
};
use partprobe_persistence_sqlite::SqliteShopSettingsRepository;
use rust_decimal::Decimal;

use crate::analysis::trusted_recorded_at;
use crate::estimate::{pricing_policy, rate_card};

const SHOP_PROFILE_ID: &str = "default-shop";

/// Host-owned Settings service. Paths and SQLite handles never cross the desktop contract.
pub struct DesktopSettingsState {
    application: Mutex<Option<ShopSettingsApplication<SqliteShopSettingsRepository>>>,
}

impl DesktopSettingsState {
    pub fn open(database_path: &Path) -> Result<Self, HostCommandError> {
        let repository = SqliteShopSettingsRepository::open(database_path)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-OPEN"))?;
        Ok(Self {
            application: Mutex::new(Some(ShopSettingsApplication::new(repository))),
        })
    }

    #[must_use]
    pub const fn unavailable() -> Self {
        Self {
            application: Mutex::new(None),
        }
    }

    pub fn load(&self) -> Result<ShopSettingsState, HostCommandError> {
        let profile = profile_id()?;
        let application = self
            .application
            .lock()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-LOCK"))?;
        let application = application.as_ref().ok_or_else(|| {
            HostCommandError::settings_unavailable("USE2-SETTINGS-NOT-CONFIGURED")
        })?;
        match application
            .load(&profile)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-LOAD"))?
        {
            ShopSettingsLoadState::NotConfigured => Ok(ShopSettingsState::NotConfigured),
            ShopSettingsLoadState::Available(draft) => Ok(ShopSettingsState::Available {
                settings: Box::new(snapshot(&draft)?),
            }),
        }
    }

    pub fn save(
        &self,
        request: &SaveShopSettingsRequest,
    ) -> Result<ShopSettingsState, HostCommandError> {
        if !request.rates.confirmed_for_session || !request.pricing.confirmed_for_session {
            return Err(HostCommandError::invalid_settings_input(
                "USE2-SETTINGS-CONFIRMATION",
            ));
        }
        if request.rates.currency != "USD" {
            return Err(HostCommandError::invalid_settings_input(
                "USE2-SETTINGS-CURRENCY",
            ));
        }
        let expected_revision = request
            .expected_revision
            .map(ShopSettingsRevision::new)
            .transpose()
            .map_err(|_| invalid_settings("USE2-SETTINGS-EXPECTED-REVISION"))?;
        let next_revision = request
            .expected_revision
            .map_or(Some(1), |value| value.checked_add(1))
            .ok_or_else(|| invalid_settings("USE2-SETTINGS-REVISION-OVERFLOW"))?;
        let profile = profile_id()?;
        let recorded_at = trusted_recorded_at()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-TIME"))?;
        let actor = ActorId::new(&request.changed_by)
            .map_err(|_| invalid_settings("USE2-SETTINGS-ACTOR"))?;
        let (mut rates, _) = rate_card(
            &request.rates,
            SHOP_PROFILE_ID,
            &recorded_at,
            actor.as_str(),
            "entered in a durable non-authoritative shop-settings draft",
            "confirmed for the saved draft; calculation use still requires session review",
        )
        .map_err(|_| invalid_settings("USE2-SETTINGS-RATES"))?;
        let mut pricing = pricing_policy(&request.pricing, &request.rates.currency)
            .map_err(|_| invalid_settings("USE2-SETTINGS-PRICING"))?;
        let mut resources = request
            .resources
            .as_ref()
            .map(|fields| resource_library(fields, &recorded_at, &request.rates.currency))
            .transpose()?;

        let mut application = self
            .application
            .lock()
            .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-LOCK"))?;
        let application_ref = application.as_ref().ok_or_else(|| {
            HostCommandError::settings_unavailable("USE2-SETTINGS-NOT-CONFIGURED")
        })?;
        if let ShopSettingsLoadState::Available(current) = application_ref
            .load(&profile)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-LOAD"))?
        {
            if let Some(existing) = current.rate_card()
                && rate_fields(existing)? == rate_fields(&rates)?
            {
                rates = existing.clone();
            }
            if let Some(existing) = current.pricing_policy()
                && pricing_fields(existing)? == pricing_fields(&pricing)?
            {
                pricing = existing.clone();
            }
            if let (Some(existing), Some(candidate)) =
                (current.resource_library(), resources.as_ref())
                && resource_fields(existing) == resource_fields(candidate)
            {
                resources = Some(existing.clone());
            }
        }
        let draft = ShopSettingsDraft::new_with_resources(
            profile,
            ShopSettingsRevision::new(next_revision)
                .map_err(|_| invalid_settings("USE2-SETTINGS-REVISION"))?,
            CurrencyCode::new("USD").map_err(|_| invalid_settings("USE2-SETTINGS-CURRENCY"))?,
            Some(rates),
            Some(pricing),
            resources,
            actor,
            recorded_at,
            &request.change_reason,
        )
        .map_err(|_| invalid_settings("USE2-SETTINGS-DRAFT"))?;
        application
            .as_mut()
            .ok_or_else(|| HostCommandError::settings_unavailable("USE2-SETTINGS-NOT-CONFIGURED"))?
            .save(&draft, expected_revision)
            .map_err(|error| map_store_error(error, "USE2-SETTINGS-SAVE"))?;
        Ok(ShopSettingsState::Available {
            settings: Box::new(snapshot(&draft)?),
        })
    }
}

fn profile_id() -> Result<ShopProfileId, HostCommandError> {
    ShopProfileId::new(SHOP_PROFILE_ID)
        .map_err(|_| HostCommandError::settings_unavailable("USE2-SETTINGS-PROFILE"))
}

fn snapshot(draft: &ShopSettingsDraft) -> Result<ShopSettingsSnapshot, HostCommandError> {
    Ok(ShopSettingsSnapshot {
        revision: draft.revision().value(),
        currency: draft.currency().as_str().to_owned(),
        rates: draft.rate_card().map(rate_fields).transpose()?,
        pricing: draft.pricing_policy().map(pricing_fields).transpose()?,
        resources: draft.resource_library().map(resource_fields),
        changed_by: draft.changed_by().as_str().to_owned(),
        changed_at: draft.changed_at().as_str().to_owned(),
        change_reason: draft.change_reason().to_owned(),
    })
}

fn resource_library(
    fields: &ShopResourceInputFields,
    recorded_at: &RecordedAt,
    currency: &str,
) -> Result<ShopResourceLibrary, HostCommandError> {
    if !fields.confirmed_for_draft {
        return Err(invalid_settings("USE2-RESOURCES-CONFIRMATION"));
    }
    let library_version = resource_version(&fields.library_version)?;
    let material_version = resource_version(&fields.material_version)?;
    let offer_version = resource_version(&fields.offer_version)?;
    let stock_version = resource_version(&fields.stock_profile_version)?;
    let machine_version = resource_version(&fields.machine_version)?;
    let runtime_version = resource_version(&fields.runtime_profile_version)?;
    let material_id = MaterialDefinitionId::new(&fields.material_id)
        .map_err(|_| invalid_settings("USE2-RESOURCES-MATERIAL-ID"))?;
    let machine_id = MachineProfileId::new(&fields.machine_id)
        .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE-ID"))?;
    let currency =
        CurrencyCode::new(currency).map_err(|_| invalid_settings("USE2-RESOURCES-CURRENCY"))?;
    let material = MaterialDefinition::new(
        material_id.clone(),
        material_version,
        &fields.material_family,
        &fields.material_grade,
        optional_text(&fields.optional_material_specification),
        optional_text(&fields.optional_material_condition),
        DensityKilogramsPerCubicMeter::new(decimal(&fields.density_kg_per_m3)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-DENSITY"))?,
        manual_source(&fields.material_source, recorded_at)?,
        LibraryRecordState::Draft,
    )
    .map_err(|_| invalid_settings("USE2-RESOURCES-MATERIAL"))?;
    let offer = MaterialOffer::new(
        MaterialOfferId::new(&fields.offer_id)
            .map_err(|_| invalid_settings("USE2-RESOURCES-OFFER-ID"))?,
        offer_version,
        material_id.clone(),
        material_version,
        &fields.supplier,
        Money::new(decimal(&fields.material_price_per_kg)?, currency.clone()),
        EffectiveDate::new(&fields.offer_effective_on)
            .map_err(|_| invalid_settings("USE2-RESOURCES-OFFER-DATE"))?,
        manual_source(&fields.offer_source, recorded_at)?,
        LibraryRecordState::Draft,
    )
    .map_err(|_| invalid_settings("USE2-RESOURCES-OFFER"))?;
    let stock = StockAllowanceProfile::new(
        StockAllowanceProfileId::new(&fields.stock_profile_id)
            .map_err(|_| invalid_settings("USE2-RESOURCES-STOCK-ID"))?,
        stock_version,
        stock_form(&fields.stock_form)?,
        StockAllowanceMillimeters::new(decimal(&fields.stock_allowance_x_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-STOCK-X"))?,
        StockAllowanceMillimeters::new(decimal(&fields.stock_allowance_y_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-STOCK-Y"))?,
        StockAllowanceMillimeters::new(decimal(&fields.stock_allowance_z_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-STOCK-Z"))?,
        manual_source(&fields.stock_source, recorded_at)?,
        LibraryRecordState::Draft,
    );
    let machine = MachineProfile::new(
        machine_id.clone(),
        machine_version,
        &fields.machine_name,
        process_class(&fields.process_class)?,
        MachineEnvelopeMillimeters::new(decimal(&fields.machine_envelope_x_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE-X"))?,
        MachineEnvelopeMillimeters::new(decimal(&fields.machine_envelope_y_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE-Y"))?,
        MachineEnvelopeMillimeters::new(decimal(&fields.machine_envelope_z_mm)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE-Z"))?,
        manual_source(&fields.machine_source, recorded_at)?,
        LibraryRecordState::Draft,
    )
    .map_err(|_| invalid_settings("USE2-RESOURCES-MACHINE"))?;
    let runtime = CoarseRuntimeProfile::new(
        RuntimeProfileId::new(&fields.runtime_profile_id)
            .map_err(|_| invalid_settings("USE2-RESOURCES-RUNTIME-ID"))?,
        runtime_version,
        machine_id,
        machine_version,
        material_id,
        material_version,
        RemovalRateCubicMillimetersPerMinute::new(decimal(&fields.removal_rate_mm3_per_minute)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-REMOVAL-RATE"))?,
        RuntimeMinutes::new(decimal(&fields.setup_minutes)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-SETUP-TIME"))?,
        RuntimeMinutes::new(decimal(&fields.programming_minutes)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-PROGRAMMING-TIME"))?,
        RuntimeMinutes::new(decimal(&fields.load_unload_minutes)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-LOAD-TIME"))?,
        RuntimeMinutes::new(decimal(&fields.inspection_minutes)?)
            .map_err(|_| invalid_settings("USE2-RESOURCES-INSPECTION-TIME"))?,
        manual_source(&fields.runtime_source, recorded_at)?,
        LibraryRecordState::Draft,
    );
    ShopResourceLibrary::new(
        ShopResourceLibraryId::new(&fields.library_id)
            .map_err(|_| invalid_settings("USE2-RESOURCES-LIBRARY-ID"))?,
        library_version,
        currency,
        material,
        offer,
        stock,
        machine,
        runtime,
    )
    .map_err(|_| invalid_settings("USE2-RESOURCES-LIBRARY"))
}

fn resource_fields(library: &ShopResourceLibrary) -> ShopResourceInputFields {
    let material = library.material();
    let offer = library.material_offer();
    let stock = library.stock_allowance();
    let machine = library.machine();
    let runtime = library.runtime();
    ShopResourceInputFields {
        confirmed_for_draft: false,
        library_id: library.id().as_str().to_owned(),
        library_version: library.version().value().to_string(),
        material_id: material.id().as_str().to_owned(),
        material_version: material.version().value().to_string(),
        material_family: material.family().to_owned(),
        material_grade: material.grade().to_owned(),
        optional_material_specification: material.specification().unwrap_or_default().to_owned(),
        optional_material_condition: material.condition().unwrap_or_default().to_owned(),
        density_kg_per_m3: decimal_text(material.density_kg_per_m3().value()),
        material_source: material.source().source_id().to_owned(),
        offer_id: offer.id().as_str().to_owned(),
        offer_version: offer.version().value().to_string(),
        supplier: offer.supplier().to_owned(),
        material_price_per_kg: decimal_text(offer.price_per_kg().amount()),
        offer_effective_on: offer.effective_from().as_str().to_owned(),
        offer_source: offer.source().source_id().to_owned(),
        stock_profile_id: stock.id().as_str().to_owned(),
        stock_profile_version: stock.version().value().to_string(),
        stock_form: stock_form_text(stock.stock_form()).to_owned(),
        stock_allowance_x_mm: decimal_text(stock.x_allowance_mm().value()),
        stock_allowance_y_mm: decimal_text(stock.y_allowance_mm().value()),
        stock_allowance_z_mm: decimal_text(stock.z_allowance_mm().value()),
        stock_source: stock.source().source_id().to_owned(),
        machine_id: machine.id().as_str().to_owned(),
        machine_version: machine.version().value().to_string(),
        machine_name: machine.name().to_owned(),
        process_class: process_class_text(machine.process_class()).to_owned(),
        machine_envelope_x_mm: decimal_text(machine.envelope_x_mm().value()),
        machine_envelope_y_mm: decimal_text(machine.envelope_y_mm().value()),
        machine_envelope_z_mm: decimal_text(machine.envelope_z_mm().value()),
        machine_source: machine.source().source_id().to_owned(),
        runtime_profile_id: runtime.id().as_str().to_owned(),
        runtime_profile_version: runtime.version().value().to_string(),
        removal_rate_mm3_per_minute: decimal_text(runtime.removal_rate_mm3_per_minute().value()),
        setup_minutes: decimal_text(runtime.setup_minutes().value()),
        programming_minutes: decimal_text(runtime.programming_minutes().value()),
        load_unload_minutes: decimal_text(runtime.load_unload_minutes().value()),
        inspection_minutes: decimal_text(runtime.inspection_minutes().value()),
        runtime_source: runtime.source().source_id().to_owned(),
    }
}

fn decimal(value: &str) -> Result<Decimal, HostCommandError> {
    Decimal::from_str(value).map_err(|_| invalid_settings("USE2-RESOURCES-DECIMAL"))
}

fn resource_version(value: &str) -> Result<ShopResourceVersion, HostCommandError> {
    value
        .parse::<u32>()
        .map_err(|_| invalid_settings("USE2-RESOURCES-VERSION"))
        .and_then(|value| {
            ShopResourceVersion::new(value).map_err(|_| invalid_settings("USE2-RESOURCES-VERSION"))
        })
}

fn optional_text(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_owned())
}

fn manual_source(value: &str, recorded_at: &RecordedAt) -> Result<SourceRef, HostCommandError> {
    SourceRef::new(SourceKind::Manual, value, None, Some(recorded_at.clone()))
        .map_err(|_| invalid_settings("USE2-RESOURCES-SOURCE"))
}

fn stock_form(value: &str) -> Result<StockForm, HostCommandError> {
    match value {
        "rectangular" => Ok(StockForm::Rectangular),
        "round" => Ok(StockForm::Round),
        "plate" => Ok(StockForm::Plate),
        _ => Err(invalid_settings("USE2-RESOURCES-STOCK-FORM")),
    }
}

const fn stock_form_text(value: StockForm) -> &'static str {
    match value {
        StockForm::Rectangular => "rectangular",
        StockForm::Round => "round",
        StockForm::Plate => "plate",
    }
}

fn process_class(value: &str) -> Result<ProcessClass, HostCommandError> {
    match value {
        "milling" => Ok(ProcessClass::Milling),
        "turning" => Ok(ProcessClass::Turning),
        "sawing" => Ok(ProcessClass::Sawing),
        "inspection" => Ok(ProcessClass::Inspection),
        _ => Err(invalid_settings("USE2-RESOURCES-PROCESS")),
    }
}

const fn process_class_text(value: ProcessClass) -> &'static str {
    match value {
        ProcessClass::Milling => "milling",
        ProcessClass::Turning => "turning",
        ProcessClass::Sawing => "sawing",
        ProcessClass::Inspection => "inspection",
    }
}

fn rate_fields(card: &RateCard) -> Result<DeveloperRateInputFields, HostCommandError> {
    let effective_on = common_effective_date(card)?;
    Ok(DeveloperRateInputFields {
        confirmed_for_session: false,
        rate_card_id: card.id().as_str().to_owned(),
        rate_card_version: card.version().value().to_string(),
        effective_on,
        currency: card.currency().as_str().to_owned(),
        setup_labor_per_hour: rate_amount(card, CostCategory::SetupLabor)?,
        programming_per_hour: rate_amount(card, CostCategory::Programming)?,
        run_labor_per_hour: rate_amount(card, CostCategory::RunLabor)?,
        machine_per_hour: rate_amount(card, CostCategory::Machine)?,
        quality_inspection_per_hour: rate_amount(card, CostCategory::QualityInspection)?,
    })
}

fn pricing_fields(
    pricing: &partprobe_domain::PricingPolicy,
) -> Result<DeveloperPricingInputFields, HostCommandError> {
    let markup_rate = match pricing.method() {
        PricingMethod::Markup { rate } => decimal_text(*rate),
        PricingMethod::TargetMargin { .. } => {
            return Err(HostCommandError::settings_unavailable(
                "USE2-SETTINGS-UNSUPPORTED-PRICING",
            ));
        }
    };
    Ok(DeveloperPricingInputFields {
        confirmed_for_session: false,
        pricing_policy_id: pricing.id().as_str().to_owned(),
        pricing_policy_version: pricing.version().value().to_string(),
        markup_rate,
        optional_price_floor: pricing
            .floor()
            .map_or_else(String::new, |value| decimal_text(value.amount())),
        optional_minimum_order: pricing
            .minimum_order()
            .map_or_else(String::new, |value| decimal_text(value.amount())),
        rounding_decimal_places: pricing.quote_total_rounding().scale().to_string(),
    })
}

fn common_effective_date(card: &RateCard) -> Result<String, HostCommandError> {
    let first = card
        .entries()
        .first()
        .ok_or_else(|| HostCommandError::settings_unavailable("USE2-SETTINGS-EMPTY-RATES"))?
        .effective_from();
    if card
        .entries()
        .iter()
        .any(|entry| entry.effective_from() != first)
    {
        return Err(HostCommandError::settings_unavailable(
            "USE2-SETTINGS-MIXED-EFFECTIVE-DATES",
        ));
    }
    Ok(first.as_str().to_owned())
}

fn rate_amount(card: &RateCard, category: CostCategory) -> Result<String, HostCommandError> {
    let mut matches = card
        .entries()
        .iter()
        .filter(|entry| entry.category() == category);
    let amount = matches
        .next()
        .ok_or_else(|| HostCommandError::settings_unavailable("USE2-SETTINGS-MISSING-RATE"))?
        .amount()
        .amount();
    if matches.next().is_some() {
        return Err(HostCommandError::settings_unavailable(
            "USE2-SETTINGS-AMBIGUOUS-RATE",
        ));
    }
    Ok(decimal_text(amount))
}

fn decimal_text(value: Decimal) -> String {
    partprobe_domain::decimal_serde::canonical(value).to_string()
}

fn map_store_error(error: ShopSettingsStoreError, diagnostic: &str) -> HostCommandError {
    match error {
        ShopSettingsStoreError::RevisionConflict => HostCommandError::settings_conflict(diagnostic),
        ShopSettingsStoreError::IntegrityConflict => {
            HostCommandError::settings_version_conflict(diagnostic)
        }
        ShopSettingsStoreError::InvalidStoredRecord
        | ShopSettingsStoreError::UnsupportedSchema
        | ShopSettingsStoreError::Unavailable => HostCommandError::settings_unavailable(diagnostic),
    }
}

fn invalid_settings(diagnostic: &str) -> HostCommandError {
    HostCommandError::invalid_settings_input(diagnostic)
}

#[cfg(test)]
mod tests {
    use partprobe_desktop_contract::HostErrorCode;
    use partprobe_test_support::TestDirectory;

    use super::*;

    fn request(expected_revision: Option<u32>) -> SaveShopSettingsRequest {
        SaveShopSettingsRequest {
            expected_revision,
            changed_by: "test-operator".to_owned(),
            change_reason: "reviewed test values".to_owned(),
            rates: DeveloperRateInputFields {
                confirmed_for_session: true,
                rate_card_id: "test-shop-rates".to_owned(),
                rate_card_version: "1".to_owned(),
                effective_on: "2026-09-04".to_owned(),
                currency: "USD".to_owned(),
                setup_labor_per_hour: "75".to_owned(),
                programming_per_hour: "90".to_owned(),
                run_labor_per_hour: "65".to_owned(),
                machine_per_hour: "110".to_owned(),
                quality_inspection_per_hour: "80".to_owned(),
            },
            pricing: DeveloperPricingInputFields {
                confirmed_for_session: true,
                pricing_policy_id: "test-shop-pricing".to_owned(),
                pricing_policy_version: "1".to_owned(),
                markup_rate: "0.25".to_owned(),
                optional_price_floor: String::new(),
                optional_minimum_order: "100".to_owned(),
                rounding_decimal_places: "2".to_owned(),
            },
            resources: None,
        }
    }

    fn resources() -> ShopResourceInputFields {
        ShopResourceInputFields {
            confirmed_for_draft: true,
            library_id: "test-resources".to_owned(),
            library_version: "1".to_owned(),
            material_id: "al-6061-t6".to_owned(),
            material_version: "1".to_owned(),
            material_family: "Aluminum".to_owned(),
            material_grade: "6061".to_owned(),
            optional_material_specification: "ASTM B221".to_owned(),
            optional_material_condition: "T6".to_owned(),
            density_kg_per_m3: "2700".to_owned(),
            material_source: "test-material-handbook".to_owned(),
            offer_id: "test-al-offer".to_owned(),
            offer_version: "1".to_owned(),
            supplier: "Synthetic supplier".to_owned(),
            material_price_per_kg: "8.50".to_owned(),
            offer_effective_on: "2026-09-09".to_owned(),
            offer_source: "test-supplier-quote".to_owned(),
            stock_profile_id: "test-rectangular-stock".to_owned(),
            stock_profile_version: "1".to_owned(),
            stock_form: "rectangular".to_owned(),
            stock_allowance_x_mm: "3".to_owned(),
            stock_allowance_y_mm: "3".to_owned(),
            stock_allowance_z_mm: "2".to_owned(),
            stock_source: "test-shop-policy".to_owned(),
            machine_id: "test-vmc".to_owned(),
            machine_version: "1".to_owned(),
            machine_name: "Synthetic VMC".to_owned(),
            process_class: "milling".to_owned(),
            machine_envelope_x_mm: "762".to_owned(),
            machine_envelope_y_mm: "508".to_owned(),
            machine_envelope_z_mm: "508".to_owned(),
            machine_source: "test-machine-manual".to_owned(),
            runtime_profile_id: "test-vmc-al-runtime".to_owned(),
            runtime_profile_version: "1".to_owned(),
            removal_rate_mm3_per_minute: "16000".to_owned(),
            setup_minutes: "60".to_owned(),
            programming_minutes: "45".to_owned(),
            load_unload_minutes: "3".to_owned(),
            inspection_minutes: "15".to_owned(),
            runtime_source: "test-estimator-review".to_owned(),
        }
    }

    #[test]
    fn first_run_save_reopen_and_stale_writer_are_explicit() {
        let directory = TestDirectory::create("desktop-settings").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let state = DesktopSettingsState::open(&database).unwrap();
        assert_eq!(state.load().unwrap(), ShopSettingsState::NotConfigured);

        let saved = state.save(&request(None)).unwrap();
        let ShopSettingsState::Available { settings } = saved else {
            panic!("saved settings must be available");
        };
        assert_eq!(settings.revision, 1);
        assert_eq!(settings.rates.as_ref().unwrap().machine_per_hour, "110");
        assert!(!settings.rates.as_ref().unwrap().confirmed_for_session);
        assert!(!settings.pricing.as_ref().unwrap().confirmed_for_session);
        assert_eq!(settings.changed_by, "test-operator");
        assert_eq!(
            state.save(&request(None)).unwrap_err().code,
            HostErrorCode::SettingsConflict
        );
        let mut changed_without_version = request(Some(1));
        changed_without_version.rates.machine_per_hour = "111".to_owned();
        assert_eq!(
            state.save(&changed_without_version).unwrap_err().code,
            HostErrorCode::SettingsVersionConflict
        );

        drop(state);
        let reopened = DesktopSettingsState::open(&database).unwrap();
        let loaded = reopened.load().unwrap();
        assert_eq!(
            serde_json::to_string(&loaded).unwrap(),
            serde_json::to_string(&ShopSettingsState::Available { settings }).unwrap()
        );
        drop(reopened);
        directory.cleanup().unwrap();
    }

    #[test]
    fn save_rejects_unconfirmed_or_non_usd_values_without_mutation() {
        let directory = TestDirectory::create("desktop-settings-invalid").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let state = DesktopSettingsState::open(&database).unwrap();

        let mut unconfirmed = request(None);
        unconfirmed.rates.confirmed_for_session = false;
        assert_eq!(
            state.save(&unconfirmed).unwrap_err().code,
            HostErrorCode::InvalidSettingsInput
        );
        let mut wrong_currency = request(None);
        wrong_currency.rates.currency = "EUR".to_owned();
        assert_eq!(
            state.save(&wrong_currency).unwrap_err().code,
            HostErrorCode::InvalidSettingsInput
        );
        let mut unconfirmed_resources = request(None);
        let mut resource_fields = resources();
        resource_fields.confirmed_for_draft = false;
        unconfirmed_resources.resources = Some(resource_fields);
        assert_eq!(
            state.save(&unconfirmed_resources).unwrap_err().code,
            HostErrorCode::InvalidSettingsInput
        );
        assert_eq!(state.load().unwrap(), ShopSettingsState::NotConfigured);

        drop(state);
        directory.cleanup().unwrap();
    }

    #[test]
    fn path_free_snapshot_preserves_missing_libraries_as_missing() {
        let draft = ShopSettingsDraft::new(
            ShopProfileId::new(SHOP_PROFILE_ID).unwrap(),
            ShopSettingsRevision::new(1).unwrap(),
            CurrencyCode::new("USD").unwrap(),
            None,
            None,
            ActorId::new("test-operator").unwrap(),
            partprobe_domain::RecordedAt::new("2026-09-04T12:00:00Z").unwrap(),
            "partial first-run draft",
        )
        .unwrap();

        let value = snapshot(&draft).unwrap();
        assert_eq!(value.currency, "USD");
        assert!(value.rates.is_none());
        assert!(value.pricing.is_none());
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(serialized.contains("\"rates\":null"));
        assert!(serialized.contains("\"pricing\":null"));
        assert!(!serialized.contains("path"));
    }

    #[test]
    fn resource_bundle_reuses_unchanged_evidence_and_rejects_changed_child_version() {
        let directory = TestDirectory::create("desktop-settings-resources").unwrap();
        let database = directory.path().join("settings.sqlite3");
        let state = DesktopSettingsState::open(&database).unwrap();
        let mut initial = request(None);
        initial.resources = Some(resources());
        let saved = state.save(&initial).unwrap();
        let ShopSettingsState::Available {
            settings: first_settings,
        } = saved
        else {
            panic!("saved settings must be available");
        };
        let resource_fields = first_settings.resources.as_ref().unwrap();
        assert_eq!(resource_fields.material_grade, "6061");
        assert_eq!(resource_fields.stock_form, "rectangular");
        assert_eq!(resource_fields.process_class, "milling");
        assert!(!resource_fields.confirmed_for_draft);

        let mut unchanged = request(Some(1));
        unchanged.change_reason = "reviewed unchanged values for a new draft".to_owned();
        unchanged.resources = Some(resources());
        let unchanged_saved = state.save(&unchanged).unwrap();
        let ShopSettingsState::Available { settings } = unchanged_saved else {
            panic!("saved settings must be available");
        };
        assert_eq!(settings.revision, 2);

        let mut divergent = request(Some(2));
        let mut changed_resources = resources();
        changed_resources.library_version = "2".to_owned();
        changed_resources.density_kg_per_m3 = "2710".to_owned();
        divergent.resources = Some(changed_resources);
        assert_eq!(
            state.save(&divergent).unwrap_err().code,
            HostErrorCode::SettingsVersionConflict
        );

        drop(state);
        let reopened = DesktopSettingsState::open(&database).unwrap();
        assert_eq!(
            reopened.load().unwrap(),
            ShopSettingsState::Available { settings }
        );
        drop(reopened);
        directory.cleanup().unwrap();
    }
}
