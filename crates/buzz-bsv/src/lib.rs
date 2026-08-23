//! Bitcoin SV integration primitives for ST8WRX.
//!
//! BSV is deliberately kept off Buzz's relay hot path. Buzz remains the live
//! collaboration/event layer; this crate provides deterministic provenance and
//! settlement primitives for economically meaningful records.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const EVENT_DOMAIN: &[u8] = b"ST8WRX/EVENT/v1\0";
const NODE_DOMAIN: &[u8] = b"ST8WRX/MERKLE/v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BsvNetwork {
    Mainnet,
    Testnet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BsvConfig {
    pub enabled: bool,
    pub network: BsvNetwork,
    /// ARC-compatible endpoint. Kept as configuration data at this layer so
    /// callers are not coupled to one broadcaster implementation.
    pub arc_url: Option<String>,
    pub max_batch_size: usize,
}

impl Default for BsvConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            network: BsvNetwork::Testnet,
            arc_url: None,
            max_batch_size: 512,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCommitment {
    /// Buzz/Nostr event id (hex SHA-256).
    pub event_id: String,
    /// Stable project/community scope. It participates in the leaf commitment,
    /// preventing the same event id from being silently re-contextualized.
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorBatch {
    pub version: u8,
    pub merkle_root: String,
    pub leaf_count: usize,
    pub event_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub siblings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorReceipt {
    pub version: u8,
    pub merkle_root: String,
    pub txid: String,
    pub network: BsvNetwork,
    pub leaf_count: usize,
}

#[derive(Debug, Error)]
pub enum BsvError {
    #[error("anchor batch cannot be empty")]
    EmptyBatch,
    #[error("event id must be a 32-byte hex digest: {0}")]
    InvalidEventId(String),
    #[error("scope cannot be empty")]
    EmptyScope,
    #[error("batch size {actual} exceeds configured maximum {max}")]
    BatchTooLarge { actual: usize, max: usize },
    #[error("leaf index {index} is out of range for batch size {len}")]
    InvalidLeafIndex { index: usize, len: usize },
    #[error("invalid proof hash: {0}")]
    InvalidProofHash(String),
    #[error("broadcast failed: {0}")]
    Broadcast(String),
}

/// Build a deterministic, domain-separated Merkle commitment over Buzz events.
///
/// The raw Nostr event id is not used directly as a Merkle leaf. The leaf hash
/// commits to a protocol domain and project scope, preventing cross-protocol or
/// cross-project reinterpretation of the same 32-byte id.
pub fn build_anchor_batch(
    commitments: &[EventCommitment],
    max_batch_size: usize,
) -> Result<AnchorBatch, BsvError> {
    let leaves = commitment_leaves(commitments, max_batch_size)?;
    let root = merkle_root(leaves);

    Ok(AnchorBatch {
        version: 1,
        merkle_root: hex::encode(root),
        leaf_count: commitments.len(),
        event_ids: commitments.iter().map(|c| c.event_id.clone()).collect(),
    })
}

pub fn build_merkle_proof(
    commitments: &[EventCommitment],
    max_batch_size: usize,
    leaf_index: usize,
) -> Result<MerkleProof, BsvError> {
    let mut level = commitment_leaves(commitments, max_batch_size)?;
    if leaf_index >= level.len() {
        return Err(BsvError::InvalidLeafIndex {
            index: leaf_index,
            len: level.len(),
        });
    }

    let mut siblings = Vec::new();
    let mut index = leaf_index;

    while level.len() > 1 {
        let sibling_index = if index % 2 == 0 {
            (index + 1).min(level.len() - 1)
        } else {
            index - 1
        };
        siblings.push(hex::encode(level[sibling_index]));

        level = next_level(&level);
        index /= 2;
    }

    Ok(MerkleProof {
        leaf_index,
        siblings,
    })
}

pub fn verify_merkle_proof(
    commitment: &EventCommitment,
    proof: &MerkleProof,
    expected_root: &str,
) -> Result<bool, BsvError> {
    let mut hash = commitment_leaf(commitment)?;
    let mut index = proof.leaf_index;

    for sibling_hex in &proof.siblings {
        let sibling = decode_hash(sibling_hex)
            .map_err(|_| BsvError::InvalidProofHash(sibling_hex.clone()))?;
        hash = if index % 2 == 0 {
            hash_node(hash, sibling)
        } else {
            hash_node(sibling, hash)
        };
        index /= 2;
    }

    Ok(hex::encode(hash).eq_ignore_ascii_case(expected_root))
}

fn commitment_leaves(
    commitments: &[EventCommitment],
    max_batch_size: usize,
) -> Result<Vec<[u8; 32]>, BsvError> {
    if commitments.is_empty() {
        return Err(BsvError::EmptyBatch);
    }
    if commitments.len() > max_batch_size {
        return Err(BsvError::BatchTooLarge {
            actual: commitments.len(),
            max: max_batch_size,
        });
    }

    commitments.iter().map(commitment_leaf).collect()
}

fn commitment_leaf(commitment: &EventCommitment) -> Result<[u8; 32], BsvError> {
    if commitment.scope.trim().is_empty() {
        return Err(BsvError::EmptyScope);
    }
    let event_id = decode_hash(&commitment.event_id)
        .map_err(|_| BsvError::InvalidEventId(commitment.event_id.clone()))?;

    let mut hasher = Sha256::new();
    hasher.update(EVENT_DOMAIN);
    hasher.update((commitment.scope.len() as u64).to_be_bytes());
    hasher.update(commitment.scope.as_bytes());
    hasher.update(event_id);
    Ok(hasher.finalize().into())
}

fn merkle_root(mut level: Vec<[u8; 32]>) -> [u8; 32] {
    while level.len() > 1 {
        level = next_level(&level);
    }
    level[0]
}

fn next_level(level: &[[u8; 32]]) -> Vec<[u8; 32]> {
    let mut next = Vec::with_capacity(level.len().div_ceil(2));
    for pair in level.chunks(2) {
        let left = pair[0];
        let right = if pair.len() == 2 { pair[1] } else { pair[0] };
        next.push(hash_node(left, right));
    }
    next
}

fn hash_node(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut first = Sha256::new();
    first.update(NODE_DOMAIN);
    first.update(left);
    first.update(right);
    let first = first.finalize();
    Sha256::digest(first).into()
}

fn decode_hash(value: &str) -> Result<[u8; 32], ()> {
    let bytes = hex::decode(value).map_err(|_| ())?;
    bytes.try_into().map_err(|_| ())
}

/// Abstracts construction/broadcasting from Buzz. Production implementations
/// can use the current BSV SDK and ARC/Teranode-compatible providers while
/// tests and callers remain provider-independent.
pub trait AnchorBroadcaster: Send + Sync {
    fn broadcast_anchor(&self, batch: &AnchorBatch) -> Result<AnchorReceipt, BsvError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(byte: &str, scope: &str) -> EventCommitment {
        EventCommitment {
            event_id: byte.repeat(32),
            scope: scope.to_string(),
        }
    }

    #[test]
    fn same_event_in_different_projects_has_different_leaf_and_root() {
        let a = build_anchor_batch(&[event("11", "project-a")], 10).unwrap();
        let b = build_anchor_batch(&[event("11", "project-b")], 10).unwrap();
        assert_ne!(a.merkle_root, b.merkle_root);
    }

    #[test]
    fn proof_round_trip_for_odd_batch() {
        let commitments = vec![
            event("11", "p"),
            event("22", "p"),
            event("33", "p"),
        ];
        let batch = build_anchor_batch(&commitments, 10).unwrap();
        for index in 0..commitments.len() {
            let proof = build_merkle_proof(&commitments, 10, index).unwrap();
            assert!(verify_merkle_proof(&commitments[index], &proof, &batch.merkle_root).unwrap());
        }
    }

    #[test]
    fn proof_fails_if_project_scope_is_changed() {
        let commitments = vec![event("11", "project-a"), event("22", "project-a")];
        let batch = build_anchor_batch(&commitments, 10).unwrap();
        let proof = build_merkle_proof(&commitments, 10, 0).unwrap();
        let replayed = event("11", "project-b");
        assert!(!verify_merkle_proof(&replayed, &proof, &batch.merkle_root).unwrap());
    }

    #[test]
    fn proof_fails_if_sibling_is_tampered() {
        let commitments = vec![event("11", "p"), event("22", "p")];
        let batch = build_anchor_batch(&commitments, 10).unwrap();
        let mut proof = build_merkle_proof(&commitments, 10, 0).unwrap();
        proof.siblings[0] = "ff".repeat(32);
        assert!(!verify_merkle_proof(&commitments[0], &proof, &batch.merkle_root).unwrap());
    }

    #[test]
    fn proof_rejects_malformed_sibling() {
        let commitment = event("11", "p");
        let proof = MerkleProof {
            leaf_index: 0,
            siblings: vec!["not-a-hash".to_string()],
        };
        assert!(matches!(
            verify_merkle_proof(&commitment, &proof, &"00".repeat(32)),
            Err(BsvError::InvalidProofHash(_))
        ));
    }

    #[test]
    fn rejects_empty_scope() {
        let result = build_anchor_batch(&[event("11", "")], 10);
        assert!(matches!(result, Err(BsvError::EmptyScope)));
    }

    #[test]
    fn rejects_oversized_batch() {
        let commitments = vec![event("22", "p"), event("33", "p")];
        assert!(matches!(
            build_anchor_batch(&commitments, 1),
            Err(BsvError::BatchTooLarge { .. })
        ));
    }
}
