//! Common types and constants for Red Packet contracts.
//!
//! # Cell Data Layout (RedPacketCellData) — 19 bytes total
//!
//! | Offset | Size | Field            | Description                        |
//! |--------|------|------------------|------------------------------------|
//! | 0      | 8    | total_amount     | Total distributable CKB (shannons) |
//! | 8      | 1    | total_count      | Total number of shares (1–255)     |
//! | 9      | 1    | remaining_count  | Shares not yet claimed             |
//! | 10     | 1    | mode             | 0 = equal, 1 = random              |
//! | 11     | 8    | expiry_since     | Expiry condition (CKB since)       |
//!
//! # Witness Layout (in WitnessArgs.lock)
//!
//! ## ClaimAction (item_id = 0) — 4 + 105 = 109 bytes
//! | Offset | Size | Field             |
//! |--------|------|-------------------|
//! | 0      | 4    | item_id (0u32 LE) |
//! | 4      | 65   | signature         |
//! | 69     | 32   | claimer_lock_hash |
//! | 101    | 8    | amount            |
//!
//! ## RefundAction (item_id = 1) — 4 + 65 = 69 bytes
//! | Offset | Size | Field             |
//! |--------|------|-------------------|
//! | 0      | 4    | item_id (1u32 LE) |
//! | 4      | 65   | signature         |

#![no_std]

/// Size of RedPacketCellData in bytes.
pub const RED_PACKET_DATA_SIZE: usize = 19;

/// Mode: equal distribution.
pub const MODE_EQUAL: u8 = 0;
/// Mode: random distribution.
pub const MODE_RANDOM: u8 = 1;

/// Union item_id for ClaimAction.
pub const ACTION_CLAIM: u32 = 0;
/// Union item_id for RefundAction.
pub const ACTION_REFUND: u32 = 1;

/// Claim witness total size: 4 (item_id) + 65 (sig) + 32 (lock_hash) + 8 (amount).
pub const CLAIM_WITNESS_SIZE: usize = 109;
/// Refund witness total size: 4 (item_id) + 65 (sig).
pub const REFUND_WITNESS_SIZE: usize = 69;

/// Signature size (secp256k1 recoverable).
pub const SIGNATURE_SIZE: usize = 65;

/// Minimum CKB cell capacity in shannons (61 bytes × 10^8 shannons/CKB = 6_100_000_000).
/// This is the theoretical minimum for a cell with no data.
/// For our red packet cell: 8 (capacity) + 32 (lock hash) + 32 (type hash) + 19 (data) = 91 bytes
/// Minimum capacity = 91 * 10^8 = 9_100_000_000 shannons = 91 CKB
pub const MIN_CELL_CAPACITY: u64 = 9_100_000_000;

/// Domain separator for claim authorization signatures.
pub const CLAIM_DOMAIN: &[u8] = b"RED_PACKET_CLAIM";

/// Red Packet cell data parsed from raw bytes.
#[derive(Debug, Clone, Copy)]
pub struct RedPacketData {
    pub total_amount: u64,
    pub total_count: u8,
    pub remaining_count: u8,
    pub mode: u8,
    pub expiry_since: u64,
}

impl RedPacketData {
    /// Decode from raw bytes (exactly 19 bytes).
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() != RED_PACKET_DATA_SIZE {
            return None;
        }
        let total_amount = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let total_count = data[8];
        let remaining_count = data[9];
        let mode = data[10];
        let expiry_since = u64::from_le_bytes([
            data[11], data[12], data[13], data[14], data[15], data[16], data[17], data[18],
        ]);
        Some(Self {
            total_amount,
            total_count,
            remaining_count,
            mode,
            expiry_since,
        })
    }

    /// Encode to raw bytes (19 bytes).
    pub fn to_bytes(&self) -> [u8; RED_PACKET_DATA_SIZE] {
        let mut buf = [0u8; RED_PACKET_DATA_SIZE];
        buf[0..8].copy_from_slice(&self.total_amount.to_le_bytes());
        buf[8] = self.total_count;
        buf[9] = self.remaining_count;
        buf[10] = self.mode;
        buf[11..19].copy_from_slice(&self.expiry_since.to_le_bytes());
        buf
    }
}

/// Claim action parsed from WitnessArgs.lock bytes.
#[derive(Debug)]
pub struct ClaimAction {
    pub signature: [u8; SIGNATURE_SIZE],
    pub claimer_lock_hash: [u8; 32],
    pub amount: u64,
}

impl ClaimAction {
    /// Decode from witness lock bytes (must be exactly 109 bytes).
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() != CLAIM_WITNESS_SIZE {
            return None;
        }
        let item_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if item_id != ACTION_CLAIM {
            return None;
        }
        let mut signature = [0u8; SIGNATURE_SIZE];
        signature.copy_from_slice(&data[4..69]);
        let mut claimer_lock_hash = [0u8; 32];
        claimer_lock_hash.copy_from_slice(&data[69..101]);
        let amount = u64::from_le_bytes([
            data[101], data[102], data[103], data[104], data[105], data[106], data[107], data[108],
        ]);
        Some(Self {
            signature,
            claimer_lock_hash,
            amount,
        })
    }

    /// Encode to witness lock bytes (109 bytes).
    pub fn to_bytes(&self) -> [u8; CLAIM_WITNESS_SIZE] {
        let mut buf = [0u8; CLAIM_WITNESS_SIZE];
        buf[0..4].copy_from_slice(&ACTION_CLAIM.to_le_bytes());
        buf[4..69].copy_from_slice(&self.signature);
        buf[69..101].copy_from_slice(&self.claimer_lock_hash);
        buf[101..109].copy_from_slice(&self.amount.to_le_bytes());
        buf
    }
}

/// Refund action parsed from WitnessArgs.lock bytes.
#[derive(Debug)]
pub struct RefundAction {
    pub signature: [u8; SIGNATURE_SIZE],
}

impl RefundAction {
    /// Decode from witness lock bytes (must be exactly 69 bytes).
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() != REFUND_WITNESS_SIZE {
            return None;
        }
        let item_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if item_id != ACTION_REFUND {
            return None;
        }
        let mut signature = [0u8; SIGNATURE_SIZE];
        signature.copy_from_slice(&data[4..69]);
        Some(Self { signature })
    }

    /// Encode to witness lock bytes (69 bytes).
    pub fn to_bytes(&self) -> [u8; REFUND_WITNESS_SIZE] {
        let mut buf = [0u8; REFUND_WITNESS_SIZE];
        buf[0..4].copy_from_slice(&ACTION_REFUND.to_le_bytes());
        buf[4..69].copy_from_slice(&self.signature);
        buf
    }
}

/// Extract the action item_id from witness lock bytes.
pub fn extract_action_id(data: &[u8]) -> Option<u32> {
    if data.len() < 4 {
        return None;
    }
    Some(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
}

/// Error codes shared between lock and type scripts.
pub mod error {
    // Type Script errors (1-49)
    pub const ERROR_INVALID_CELL_COUNT: i8 = 1;
    pub const ERROR_INVALID_DATA: i8 = 2;
    pub const ERROR_INVALID_INITIAL_STATE: i8 = 3;
    pub const ERROR_IMMUTABLE_FIELD_CHANGED: i8 = 4;
    pub const ERROR_INVALID_STATE_TRANSITION: i8 = 5;
    pub const ERROR_CAPACITY_MISMATCH: i8 = 6;
    pub const ERROR_CLAIMER_OUTPUT_NOT_FOUND: i8 = 7;
    pub const ERROR_INVALID_AMOUNT: i8 = 8;
    pub const ERROR_TYPE_ID: i8 = 9;
    pub const ERROR_SCRIPT_MISMATCH: i8 = 10;
    pub const ERROR_INVALID_DESTROY: i8 = 11;

    // Lock Script errors (50-99)
    pub const ERROR_INVALID_WITNESS: i8 = 50;
    pub const ERROR_INVALID_SIGNATURE: i8 = 51;
    pub const ERROR_SINCE_CHECK_FAILED: i8 = 52;
    pub const ERROR_PUBKEY_HASH_MISMATCH: i8 = 53;
    pub const ERROR_SYSCALL: i8 = 54;
}
