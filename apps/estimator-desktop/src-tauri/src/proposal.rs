use partprobe_application::{
    DeveloperEstimateInputProposal, DeveloperEstimateProposalApplication,
    DeveloperEstimateProposalReasonCode, DraftEstimateSession,
};
use partprobe_desktop_contract::{DraftEstimateProposalEvaluation, DraftEstimateProposalSummary};
use partprobe_domain::{ShopSettingsDraft, ValueState};
use rust_decimal::Decimal;

pub fn prepare_draft_estimate_proposal(
    session: &DraftEstimateSession,
    settings: &ShopSettingsDraft,
    selection_id: &str,
    analysis_id: &str,
) -> DraftEstimateProposalEvaluation {
    match DeveloperEstimateProposalApplication.propose(session.geometry(), settings) {
        ValueState::Available { value } => DraftEstimateProposalEvaluation::Available {
            proposal: Box::new(summary(&value, settings, selection_id, analysis_id)),
        },
        ValueState::Unavailable { reason } => {
            DraftEstimateProposalEvaluation::Unavailable { reason }
        }
        ValueState::Blocked { reason }
        | ValueState::Unknown { reason }
        | ValueState::Stale { reason, .. } => DraftEstimateProposalEvaluation::Blocked { reason },
    }
}

fn summary(
    proposal: &DeveloperEstimateInputProposal,
    settings: &ShopSettingsDraft,
    selection_id: &str,
    analysis_id: &str,
) -> DraftEstimateProposalSummary {
    let library = settings
        .resource_library()
        .expect("available developer proposal must retain its source resource library");
    let model = proposal.model_extents_mm();
    let blank = proposal.blank_dimensions_mm();
    let times = proposal.coarse_times();
    let (rule_major, rule_minor, rule_patch) = proposal.rule_version();
    DraftEstimateProposalSummary {
        selection_id: selection_id.to_owned(),
        analysis_id: analysis_id.to_owned(),
        settings_revision: settings.revision().value(),
        library_id: proposal.library_id().as_str().to_owned(),
        library_version: proposal.library_version().value(),
        material_id: proposal.material_id().as_str().to_owned(),
        material_version: proposal.material_version().value(),
        material_grade: library.material().grade().to_owned(),
        material_offer_id: proposal.material_offer_id().as_str().to_owned(),
        material_offer_version: proposal.material_offer_version().value(),
        stock_allowance_id: proposal.stock_allowance_id().as_str().to_owned(),
        stock_allowance_version: proposal.stock_allowance_version().value(),
        machine_id: proposal.machine_id().as_str().to_owned(),
        machine_version: proposal.machine_version().value(),
        machine_name: library.machine().name().to_owned(),
        runtime_id: proposal.runtime_id().as_str().to_owned(),
        runtime_version: proposal.runtime_version().value(),
        model_extents_mm: [
            decimal_text(model.x().value()),
            decimal_text(model.y().value()),
            decimal_text(model.z().value()),
        ],
        part_volume_mm3: decimal_text(proposal.part_volume_mm3().value()),
        blank_dimensions_mm: [
            decimal_text(blank.x().value()),
            decimal_text(blank.y().value()),
            decimal_text(blank.z().value()),
        ],
        blank_volume_mm3: decimal_text(proposal.blank_volume_mm3().value()),
        removed_volume_mm3: decimal_text(proposal.removed_volume_mm3().value()),
        material_density_kg_per_m3: decimal_text(proposal.material_density_kg_per_m3().value()),
        blank_mass_kg: decimal_text(proposal.blank_mass_kg().value()),
        material_price_per_kg: decimal_text(proposal.material_price_per_kg().amount()),
        unit_stock_material_cost: decimal_text(proposal.unit_stock_material_cost().amount()),
        currency: proposal
            .unit_stock_material_cost()
            .currency()
            .as_str()
            .to_owned(),
        removal_rate_mm3_per_minute: decimal_text(proposal.removal_rate_mm3_per_minute().value()),
        setup_hours: minutes_to_hours(times.setup().value()),
        programming_hours: minutes_to_hours(times.programming().value()),
        cutting_hours_per_item: minutes_to_hours(times.cutting().value()),
        load_unload_hours_per_item: minutes_to_hours(times.load_unload().value()),
        quality_inspection_hours: minutes_to_hours(times.quality().value()),
        rule_id: proposal.rule_id().to_owned(),
        rule_version: format!("{rule_major}.{rule_minor}.{rule_patch}"),
        reason_codes: proposal
            .reason_codes()
            .iter()
            .map(|reason| reason_code(*reason).to_owned())
            .collect(),
    }
}

fn minutes_to_hours(minutes: Decimal) -> String {
    decimal_text(minutes / Decimal::from(60_u32))
}

fn decimal_text(value: Decimal) -> String {
    value.normalize().to_string()
}

const fn reason_code(reason: DeveloperEstimateProposalReasonCode) -> &'static str {
    match reason {
        DeveloperEstimateProposalReasonCode::SessionOnlyDeveloperEvidence => {
            "session_only_developer_evidence"
        }
        DeveloperEstimateProposalReasonCode::DraftShopResourceLibrary => {
            "draft_shop_resource_library"
        }
        DeveloperEstimateProposalReasonCode::AxisAlignedOrientationOnly => {
            "axis_aligned_orientation_only"
        }
        DeveloperEstimateProposalReasonCode::StandardSizeNotResolved => {
            "standard_size_not_resolved"
        }
        DeveloperEstimateProposalReasonCode::AvailabilityNotResolved => "availability_not_resolved",
        DeveloperEstimateProposalReasonCode::CoarseRuntimeNotCam => "coarse_runtime_not_cam",
        DeveloperEstimateProposalReasonCode::HumanReviewAndAdoptionRequired => {
            "human_review_and_adoption_required"
        }
    }
}
