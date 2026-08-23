//! Bitcoin SV integration primitives for Buzz.
//!
//! The first integration keeps BSV off the relay hot path. Buzz remains the
//! live collaboration/event layer; `buzz-bsv` provides deterministic proofs
//! and settlement interfaces for economically meaningful events.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

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
    /// ARC-compatible endpoint. Kept as data only at this layer so the relay
    /// can remain independent from any single broadcaster implementation.
    pub arc_url: Option<String>,
    /// Maximum number of event commitments included in one anchor batch.
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
    /// Optional project/community scope used by the caller when assembling a batch.
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorBatch {
    pub merkle_root: String,
    pub leaf_count: usize,
    pub event_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorReceipt {
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
    #[error("batch size {actual} exceeds configured maximum {max}")]
    BatchTooLarge { actual: usize, max: usize },
    #[error("broadcast failed: {0}")]
    Broadcast(String),
}

/// Builds a deterministic Merkle commitment over Buzz event ids.
///
/// Leaves are the raw 32-byte event digests. Odd levels duplicate the final
/// hash, matching the conventional Bitcoin Merkle-tree rule.
pub fn build_anchor_batch(
    commitments: &[EventCommitment],
    max_batch_size: usize,
) -> Result<AnchorBatch, BsvError> {
    if commitments.is_empty() {
        return Err(BsvError::EmptyBatch);
    }
    if commitments.len() > max_batch_size {
        return Err(BsvError::BatchTooLarge {
            actual: commitments.len(),
            max: max_batch_size,
        });
    }

    let mut level = Vec::with_capacity(commitments.len());
    let mut event_ids = Vec::with_capacity(commitments.len());

    for commitment in commitments {
        let bytes = hex::decode(&commitment.event_id)
            .map_err(|_| BsvError::InvalidEventId(commitment.event_id.clone()))?;
        if bytes.len() != 32 {
            return Err(BsvError::InvalidEventId(commitment.event_id.clone()));
        }
        let leaf: [u8; 32] = bytes
            .try_into()
            .map_err(|_| BsvError::InvalidEventId(commitment.event_id.clone()))?;
        level.push(leaf);
        event_ids.push(commitment.event_id.clone());
    }

    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let left = pair[0];
            let right = if pair.len() == 2 { pair[1] } else { pair[0] };
            next.push(double_sha256_pair(left, right));
        }
        level = next;
    }

    Ok(AnchorBatch {
        merkle_root: hex::encode(level[0]),
        leaf_count: event_ids.len(),
        event_ids,
    })
}

fn double_sha256_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut first = Sha256::new();
    first.update(left);
    first.update(right);
    let first = first.finalize();
    let second = Sha256::digest(first);
    second.into()
}

/// Abstracts transaction construction/broadcasting from the relay. A production
/// implementation can wrap the official BSV Rust SDK plus an ARC-compatible
/// broadcaster without coupling callers to that stack.
pub trait AnchorBroadcaster: Send + Sync {
    fn broadcast_anchor(&self, batch: &AnchorBatch) -> Result<AnchorReceipt, BsvError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_leaf_root_is_event_id() {
        let id = "11".repeat(32);
        let batch = build_anchor_batch(
            &[EventCommitment {
                event_id: id.clone(),
                scope: None,
            }],
            10,
        )
        .unwrap();
        assert_eq!(batch.merkle_root, id);
        assert_eq!(batch.leaf_count, 1);
    }

    #[test]
    fn rejects_oversized_batch() {
        let commitments = vec![
            EventCommitment {
                event_id: "22".repeat(32),
                scope: None,
            },
            EventCommitment {
                event_id: "33".repeat(32),
                scope: None,
            },
        ];
        assert!(matches!(
            build_anchor_batch(&commitments, 1),
            Err(BsvError::BatchTooLarge { .. })
        ));
    }
}
