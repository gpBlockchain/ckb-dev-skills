//! Red Packet Lock Script
//!
//! Validates spending authorization for Red Packet cells:
//! - Claim mode (action_id = 0): Verify creator's offline authorization signature
//! - Refund mode (action_id = 1): Verify creator's tx signature + since time-lock
//!
//! Lock args: creator_pubkey_hash (20 bytes, blake160 of secp256k1 public key)
//!
//! ## secp256k1 verification
//!
//! Dynamically loads the CKB system secp256k1 shared library from a dep cell
//! using `ckb_std::dynamic_loading_c_impl::CKBDLContext`. Requires two system
//! dep cells in the transaction:
//!   1. The secp256k1 **code** cell (shared library ELF)
//!   2. The secp256k1 **data** cell (1,048,576 bytes precomputed tables)

#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

use alloc::vec::Vec;
use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{
    load_cell_data, load_cell_type, load_input_since, load_script, load_tx_hash,
    load_witness_args,
};
use ckb_std::since::Since;

use red_packet_common::error::*;
use red_packet_common::{
    ACTION_CLAIM, ACTION_REFUND, CLAIM_DOMAIN, ClaimAction, RedPacketData, RefundAction,
    extract_action_id,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const SIGNATURE_SIZE: usize = 65;
const HASH160_SIZE: usize = 20;
const UNCOMPRESSED_PUBKEY_SIZE: usize = 65;
const SECP256K1_DATA_SIZE: usize = 1_048_576;

/// Code hash of the CKB system secp256k1 code cell (data hash).
/// This is the well-known value on CKB mainnet (Lina) and testnet.
const SECP256K1_CODE_HASH: [u8; 32] = [
    0x9b, 0xd7, 0xe0, 0x6f, 0x3e, 0xcf, 0x4b, 0xe0, 0xf2, 0xfc, 0xd2, 0x18, 0x8b, 0x23, 0xf1,
    0xb9, 0xfc, 0xc8, 0x8e, 0x5d, 0x4b, 0x65, 0xa8, 0x63, 0x7b, 0x17, 0x72, 0x3b, 0xbd, 0xa3,
    0xcc, 0xe8,
];

/// Size of the dynamic loading context buffer.
const DL_CONTEXT_SIZE: usize = 512 * 1024;

// ---------------------------------------------------------------------------
// C function type definitions for the secp256k1 library
// ---------------------------------------------------------------------------

/// Initialize secp256k1 context with precomputed data.
type InitFn =
    unsafe extern "C" fn(context: *mut u8, precomputed_data: *const u8);

/// Recover uncompressed pubkey from signature. Returns 0 on success.
type RecoverFn = unsafe extern "C" fn(
    context: *const u8,
    pubkey_output: *mut u8,
    signature: *const u8,
    message: *const u8,
) -> i32;

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

pub fn program_entry() -> i8 {
    match main() {
        Ok(()) => 0,
        Err(code) => code,
    }
}

fn main() -> Result<(), i8> {
    // Load lock script to get creator_pubkey_hash from args
    let script = load_script().map_err(|_| ERROR_SYSCALL)?;
    let args: Vec<u8> = script.as_reader().args().raw_data().to_vec();
    if args.len() < HASH160_SIZE {
        return Err(ERROR_INVALID_WITNESS);
    }
    let mut creator_pubkey_hash = [0u8; HASH160_SIZE];
    creator_pubkey_hash.copy_from_slice(&args[..HASH160_SIZE]);

    // Load witness from group input
    let witness_args =
        load_witness_args(0, Source::GroupInput).map_err(|_| ERROR_INVALID_WITNESS)?;
    let lock_bytes: Vec<u8> = witness_args
        .lock()
        .to_opt()
        .ok_or(ERROR_INVALID_WITNESS)?
        .raw_data()
        .to_vec();

    let action_id = extract_action_id(&lock_bytes).ok_or(ERROR_INVALID_WITNESS)?;

    match action_id {
        ACTION_CLAIM => validate_claim(&lock_bytes, &creator_pubkey_hash),
        ACTION_REFUND => validate_refund(&lock_bytes, &creator_pubkey_hash),
        _ => Err(ERROR_INVALID_WITNESS),
    }
}

// ---------------------------------------------------------------------------
// Claim validation
// ---------------------------------------------------------------------------

/// Validate a claim action.
///
/// 1. Parse ClaimAction from witness
/// 2. Load type_id from current cell's type script args
/// 3. Build message: blake2b("RED_PACKET_CLAIM" || claimer_lock_hash || amount || type_id)
/// 4. Recover pubkey from signature
/// 5. Verify blake160(pubkey) == creator_pubkey_hash
fn validate_claim(lock_bytes: &[u8], creator_pubkey_hash: &[u8; HASH160_SIZE]) -> Result<(), i8> {
    let claim = ClaimAction::from_bytes(lock_bytes).ok_or(ERROR_INVALID_WITNESS)?;

    // Load type_id from the current cell's type script args
    let type_id = load_type_id_from_group_input()?;

    // Build the authorization message
    let message = build_claim_message(&claim.claimer_lock_hash, claim.amount, &type_id);

    // Verify the signature
    verify_secp256k1_blake160(&message, &claim.signature, creator_pubkey_hash)
}

// ---------------------------------------------------------------------------
// Refund validation
// ---------------------------------------------------------------------------

/// Validate a refund action.
///
/// 1. Parse RefundAction from witness
/// 2. Load cell data to get expiry_since
/// 3. Verify since >= expiry_since
/// 4. Verify creator's transaction signature (over tx_hash)
fn validate_refund(
    lock_bytes: &[u8],
    creator_pubkey_hash: &[u8; HASH160_SIZE],
) -> Result<(), i8> {
    let refund = RefundAction::from_bytes(lock_bytes).ok_or(ERROR_INVALID_WITNESS)?;

    // Load cell data to get expiry_since
    let data_raw = load_cell_data(0, Source::GroupInput).map_err(|_| ERROR_INVALID_DATA)?;
    let rp = RedPacketData::from_bytes(&data_raw).ok_or(ERROR_INVALID_DATA)?;

    // Check since >= expiry_since
    let input_since = load_input_since(0, Source::GroupInput).map_err(|_| ERROR_SYSCALL)?;
    let since = Since::new(input_since);
    let expiry = Since::new(rp.expiry_since);

    // Both must have valid flags
    if !since.flags_is_valid() || !expiry.flags_is_valid() {
        return Err(ERROR_SINCE_CHECK_FAILED);
    }

    // Compare since values - they must be comparable (same type) and since >= expiry
    match since.partial_cmp(&expiry) {
        Some(core::cmp::Ordering::Less) => return Err(ERROR_SINCE_CHECK_FAILED),
        None => return Err(ERROR_SINCE_CHECK_FAILED),
        _ => {} // Greater or Equal is OK
    }

    // Verify creator's transaction signature over tx_hash
    let tx_hash = load_tx_hash().map_err(|_| ERROR_SYSCALL)?;
    verify_secp256k1_blake160(&tx_hash, &refund.signature, creator_pubkey_hash)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load the type_id (32 bytes) from the type script args of the first group input cell.
fn load_type_id_from_group_input() -> Result<[u8; 32], i8> {
    let type_script = load_cell_type(0, Source::GroupInput)
        .map_err(|_| ERROR_SYSCALL)?
        .ok_or(ERROR_SYSCALL)?;
    let args: Vec<u8> = type_script.as_reader().args().raw_data().to_vec();
    if args.len() < 32 {
        return Err(ERROR_SYSCALL);
    }
    let mut type_id = [0u8; 32];
    type_id.copy_from_slice(&args[..32]);
    Ok(type_id)
}

/// Build the claim authorization message:
/// blake2b("RED_PACKET_CLAIM" || claimer_lock_hash || amount_le || type_id)
fn build_claim_message(
    claimer_lock_hash: &[u8; 32],
    amount: u64,
    type_id: &[u8; 32],
) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut blake2b = new_blake2b();
    blake2b.update(CLAIM_DOMAIN);
    blake2b.update(claimer_lock_hash);
    blake2b.update(&amount.to_le_bytes());
    blake2b.update(type_id);
    blake2b.finalize(&mut result);
    result
}

// ---------------------------------------------------------------------------
// secp256k1 ECDSA verification via dynamic loading
// ---------------------------------------------------------------------------

/// Recover pubkey via dynamic-loaded secp256k1, Blake2b-160 hash, compare.
///
/// Signature format: `r (32) || s (32) || recovery_id (1)` (65 bytes).
#[cfg(target_arch = "riscv64")]
fn verify_secp256k1_blake160(
    msg: &[u8; 32],
    sig: &[u8; SIGNATURE_SIZE],
    expected: &[u8; HASH160_SIZE],
) -> Result<(), i8> {
    use ckb_std::ckb_types::core::ScriptHashType;
    use ckb_std::dynamic_loading_c_impl::{CKBDLContext, Symbol};

    let rec_id = sig[64];
    if rec_id > 3 {
        return Err(ERROR_INVALID_SIGNATURE);
    }

    // Load the secp256k1 precomputed data from dep cells (1 MB).
    let secp_data = load_secp256k1_data()?;

    // Dynamically load the secp256k1 code cell.
    let mut context: CKBDLContext<[u8; DL_CONTEXT_SIZE]> = unsafe { CKBDLContext::new() };
    let lib = context
        .load_by(&SECP256K1_CODE_HASH, ScriptHashType::Data)
        .map_err(|_| ERROR_INVALID_SIGNATURE)?;

    // Look up the C functions.
    let init_fn: Symbol<InitFn> =
        unsafe { lib.get(b"ckb_secp256k1_custom_verify_only_initialize") }
            .ok_or(ERROR_INVALID_SIGNATURE)?;
    let recover_fn: Symbol<RecoverFn> = unsafe { lib.get(b"ckb_secp256k1_custom_recover") }
        .ok_or(ERROR_INVALID_SIGNATURE)?;

    // Initialize the secp256k1 context with the precomputed data.
    let mut secp_ctx = [0u8; 1024];
    unsafe {
        (*init_fn)(secp_ctx.as_mut_ptr(), secp_data.as_ptr());
    }

    // Build the C-ABI signature: [rec_id (1) | r (32) | s (32)]
    let mut sig_c = [0u8; SIGNATURE_SIZE];
    sig_c[0] = rec_id;
    sig_c[1..33].copy_from_slice(&sig[0..32]);
    sig_c[33..65].copy_from_slice(&sig[32..64]);

    // Recover the uncompressed public key.
    let mut pubkey = [0u8; UNCOMPRESSED_PUBKEY_SIZE];
    let ret = unsafe {
        (*recover_fn)(
            secp_ctx.as_ptr(),
            pubkey.as_mut_ptr(),
            sig_c.as_ptr(),
            msg.as_ptr(),
        )
    };
    if ret != 0 {
        return Err(ERROR_INVALID_SIGNATURE);
    }

    // Blake2b-160 of the recovered pubkey.
    let hash = blake2b_160(&pubkey);

    // Constant-time comparison.
    let mut diff: u8 = 0;
    let mut i = 0;
    while i < HASH160_SIZE {
        diff |= hash[i] ^ expected[i];
        i += 1;
    }
    if diff != 0 {
        return Err(ERROR_PUBKEY_HASH_MISMATCH);
    }

    Ok(())
}

/// Stub for non-RISC-V targets (tests run on x86_64).
#[cfg(not(target_arch = "riscv64"))]
fn verify_secp256k1_blake160(
    _msg: &[u8; 32],
    _sig: &[u8; SIGNATURE_SIZE],
    _expected: &[u8; HASH160_SIZE],
) -> Result<(), i8> {
    // In test/library mode, signature verification is not available.
    // Tests should use the always_success lock or mock this.
    Ok(())
}

/// Load the secp256k1 precomputed data (1,048,576 bytes) from cell deps.
fn load_secp256k1_data() -> Result<Vec<u8>, i8> {
    for i in 0.. {
        match load_cell_data(i, Source::CellDep) {
            Ok(data) => {
                if data.len() == SECP256K1_DATA_SIZE {
                    return Ok(data);
                }
            }
            Err(_) => break,
        }
    }
    Err(ERROR_INVALID_SIGNATURE)
}

// ---------------------------------------------------------------------------
// Blake2b helpers
// ---------------------------------------------------------------------------

fn new_blake2b() -> ckb_hash::Blake2b {
    ckb_hash::new_blake2b()
}

fn blake2b_160(data: &[u8]) -> [u8; HASH160_SIZE] {
    let mut h = new_blake2b();
    h.update(data);
    let mut full = [0u8; 32];
    h.finalize(&mut full);
    let mut out = [0u8; HASH160_SIZE];
    out.copy_from_slice(&full[..HASH160_SIZE]);
    out
}
