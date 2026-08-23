//! ST8WRX contribution protocol primitives.
//!
//! This crate contains no I/O and no blockchain dependency. It models the
//! evidence and governance state that higher layers can persist in Buzz and
//! selectively anchor through `buzz-bsv`.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const CONTRIBUTION_DOMAIN: &[u8] = b"ST8WRX/CONTRIBUTION/v1\0";
const DECISION_DOMAIN: &[u8] = b"ST8WRX/DECISION/v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionClass {
    Intellectual,
    Architecture,
    Engineering,
    ProductDesign,
    AgentWork,
    Compute,
    TestingSecurityReview,
    ResearchData,
    CommercialDistribution,
    Capital,
}

impl ContributionClass {
    const fn protocol_code(self) -> u8 {
        match self {
            Self::Intellectual => 1,
            Self::Architecture => 2,
            Self::Engineering => 3,
            Self::ProductDesign => 4,
            Self::AgentWork => 5,
            Self::Compute => 6,
            Self::TestingSecurityReview => 7,
            Self::ResearchData => 8,
            Self::CommercialDistribution => 9,
            Self::Capital => 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributorKind {
    Human,
    Agent,
    ComputeNode,
    Organization,
}

impl ContributorKind {
    const fn protocol_code(self) -> u8 {
        match self {
            Self::Human => 1,
            Self::Agent => 2,
            Self::ComputeNode => 3,
            Self::Organization => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// Stable evidence type such as `nostr_event`, `git_commit`, `workflow_run`,
    /// `agent_turn_metric`, `mesh_job`, or `artifact`.
    pub kind: String,
    /// Type-specific immutable identifier/hash.
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributionRecord {
    /// NIP-MP project coordinate or another stable project identifier.
    pub project: String,
    /// Contributor identity. For humans/agents this is normally a Nostr pubkey;
    /// compute nodes use their independently bound node identity.
    pub contributor: String,
    pub contributor_kind: ContributorKind,
    pub class: ContributionClass,
    /// Unix timestamp when the contribution record was proposed.
    pub created_at: i64,
    /// Human-readable summary. This is evidence context, not the authoritative
    /// artifact itself.
    pub summary: String,
    /// Immutable references used to ground the claim. Digest construction treats
    /// this as a set and sorts it canonically by `(kind, reference)`.
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Accepted,
    Rejected,
    Adjusted,
}

impl DecisionStatus {
    const fn protocol_code(self) -> u8 {
        match self {
            Self::Accepted => 1,
            Self::Rejected => 2,
            Self::Adjusted => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributionDecision {
    /// Hex digest returned by `contribution_digest`.
    pub contribution_id: String,
    pub project: String,
    pub status: DecisionStatus,
    /// Non-transferable contribution units awarded by the project. Rejected
    /// decisions must award zero units.
    pub units: u64,
    /// Rule/model version used for the decision.
    pub policy_version: String,
    /// Identities that approved/adjudicated the decision. Digest construction
    /// treats this as a set and sorts it canonically.
    pub approvers: Vec<String>,
    pub decided_at: i64,
    pub rationale: String,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("project identifier cannot be empty")]
    EmptyProject,
    #[error("contributor identifier cannot be empty")]
    EmptyContributor,
    #[error("contribution summary cannot be empty")]
    EmptySummary,
    #[error("contribution must contain at least one evidence reference")]
    MissingEvidence,
    #[error("evidence kind/reference cannot be empty")]
    InvalidEvidence,
    #[error("policy version cannot be empty")]
    EmptyPolicyVersion,
    #[error("decision must include at least one approver")]
    MissingApprover,
    #[error("rejected contribution cannot award units")]
    RejectedWithUnits,
    #[error("contribution id must be a 32-byte hex digest")]
    InvalidContributionId,
}

pub fn validate_contribution(record: &ContributionRecord) -> Result<(), ProtocolError> {
    if record.project.trim().is_empty() {
        return Err(ProtocolError::EmptyProject);
    }
    if record.contributor.trim().is_empty() {
        return Err(ProtocolError::EmptyContributor);
    }
    if record.summary.trim().is_empty() {
        return Err(ProtocolError::EmptySummary);
    }
    if record.evidence.is_empty() {
        return Err(ProtocolError::MissingEvidence);
    }
    if record
        .evidence
        .iter()
        .any(|e| e.kind.trim().is_empty() || e.reference.trim().is_empty())
    {
        return Err(ProtocolError::InvalidEvidence);
    }
    Ok(())
}

pub fn validate_decision(decision: &ContributionDecision) -> Result<(), ProtocolError> {
    if hex::decode(&decision.contribution_id)
        .ok()
        .filter(|bytes| bytes.len() == 32)
        .is_none()
    {
        return Err(ProtocolError::InvalidContributionId);
    }
    if decision.project.trim().is_empty() {
        return Err(ProtocolError::EmptyProject);
    }
    if decision.policy_version.trim().is_empty() {
        return Err(ProtocolError::EmptyPolicyVersion);
    }
    if decision.approvers.is_empty() || decision.approvers.iter().any(|a| a.trim().is_empty()) {
        return Err(ProtocolError::MissingApprover);
    }
    if decision.status == DecisionStatus::Rejected && decision.units != 0 {
        return Err(ProtocolError::RejectedWithUnits);
    }
    Ok(())
}

/// Canonical v1 digest of a contribution claim.
///
/// Encoding is explicit and length-prefixed rather than JSON-based so changes
/// in map ordering or serializer behavior cannot alter protocol identities.
pub fn contribution_digest(record: &ContributionRecord) -> Result<String, ProtocolError> {
    validate_contribution(record)?;
    let mut h = Sha256::new();
    h.update(CONTRIBUTION_DOMAIN);
    put_str(&mut h, &record.project);
    put_str(&mut h, &record.contributor);
    h.update([record.contributor_kind.protocol_code()]);
    h.update([record.class.protocol_code()]);
    h.update(record.created_at.to_be_bytes());
    put_str(&mut h, &record.summary);

    let mut evidence = record.evidence.clone();
    evidence.sort_by(|a, b| (&a.kind, &a.reference).cmp(&(&b.kind, &b.reference)));
    evidence.dedup_by(|a, b| a.kind == b.kind && a.reference == b.reference);
    h.update((evidence.len() as u64).to_be_bytes());
    for item in &evidence {
        put_str(&mut h, &item.kind);
        put_str(&mut h, &item.reference);
    }
    Ok(hex::encode(h.finalize()))
}

pub fn decision_digest(decision: &ContributionDecision) -> Result<String, ProtocolError> {
    validate_decision(decision)?;
    let mut h = Sha256::new();
    h.update(DECISION_DOMAIN);
    put_str(&mut h, &decision.contribution_id);
    put_str(&mut h, &decision.project);
    h.update([decision.status.protocol_code()]);
    h.update(decision.units.to_be_bytes());
    put_str(&mut h, &decision.policy_version);

    let mut approvers = decision.approvers.clone();
    approvers.sort();
    approvers.dedup();
    h.update((approvers.len() as u64).to_be_bytes());
    for approver in &approvers {
        put_str(&mut h, approver);
    }
    h.update(decision.decided_at.to_be_bytes());
    put_str(&mut h, &decision.rationale);
    Ok(hex::encode(h.finalize()))
}

fn put_str(h: &mut Sha256, value: &str) {
    h.update((value.len() as u64).to_be_bytes());
    h.update(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contribution() -> ContributionRecord {
        ContributionRecord {
            project: "30621:alice:project-nova".into(),
            contributor: "alice".into(),
            contributor_kind: ContributorKind::Human,
            class: ContributionClass::Engineering,
            created_at: 1_777_777_777,
            summary: "Implemented authentication".into(),
            evidence: vec![EvidenceRef {
                kind: "git_commit".into(),
                reference: "abc123".into(),
            }],
        }
    }

    #[test]
    fn contribution_digest_is_deterministic() {
        let record = contribution();
        assert_eq!(contribution_digest(&record).unwrap(), contribution_digest(&record).unwrap());
    }

    #[test]
    fn contribution_digest_changes_with_project() {
        let a = contribution();
        let mut b = a.clone();
        b.project = "30621:alice:other".into();
        assert_ne!(contribution_digest(&a).unwrap(), contribution_digest(&b).unwrap());
    }

    #[test]
    fn evidence_order_does_not_change_contribution_id() {
        let mut a = contribution();
        a.evidence.push(EvidenceRef {
            kind: "nostr_event".into(),
            reference: "def456".into(),
        });
        let mut b = a.clone();
        b.evidence.reverse();
        assert_eq!(contribution_digest(&a).unwrap(), contribution_digest(&b).unwrap());
    }

    #[test]
    fn approver_order_does_not_change_decision_id() {
        let base = ContributionDecision {
            contribution_id: "11".repeat(32),
            project: "p".into(),
            status: DecisionStatus::Accepted,
            units: 10,
            policy_version: "v1".into(),
            approvers: vec!["alice".into(), "bob".into()],
            decided_at: 1,
            rationale: "accepted".into(),
        };
        let mut reversed = base.clone();
        reversed.approvers.reverse();
        assert_eq!(decision_digest(&base).unwrap(), decision_digest(&reversed).unwrap());
    }

    #[test]
    fn rejected_decision_cannot_award_units() {
        let decision = ContributionDecision {
            contribution_id: "11".repeat(32),
            project: "p".into(),
            status: DecisionStatus::Rejected,
            units: 10,
            policy_version: "v1".into(),
            approvers: vec!["bob".into()],
            decided_at: 1,
            rationale: "duplicate".into(),
        };
        assert!(matches!(validate_decision(&decision), Err(ProtocolError::RejectedWithUnits)));
    }
}
