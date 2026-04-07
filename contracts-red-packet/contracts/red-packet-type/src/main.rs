//! Red Packet Type Script
//!
//! Validates the lifecycle of Red Packet cells:
//! - Creation (0 group inputs, 1 group output): Type ID + initial state validation
//! - Claim (1 group input, 1 group output): State transition validation
//! - Destroy (1 group input, 0 group outputs): Final claim or refund

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
use ckb_std::ckb_types::prelude::*;
use ckb_std::high_level::{
    QueryIter, load_cell_capacity, load_cell_data, load_cell_lock, load_cell_lock_hash,
    load_cell_type, load_witness_args,
};
use ckb_std::type_id::check_type_id;

use red_packet_common::error::*;
use red_packet_common::{
    ClaimAction, RedPacketData, ACTION_CLAIM, MIN_CELL_CAPACITY, MODE_EQUAL, MODE_RANDOM,
    extract_action_id,
};

pub fn program_entry() -> i8 {
    match main() {
        Ok(()) => 0,
        Err(code) => code,
    }
}

fn main() -> Result<(), i8> {
    let group_input_count = count_cells(Source::GroupInput);
    let group_output_count = count_cells(Source::GroupOutput);

    match (group_input_count, group_output_count) {
        (0, 1) => validate_creation(),
        (1, 1) => validate_claim(),
        (1, 0) => validate_destroy(),
        _ => Err(ERROR_INVALID_CELL_COUNT),
    }
}

/// Count cells in the given source.
fn count_cells(source: Source) -> usize {
    QueryIter::new(load_cell_capacity, source).count()
}

/// Validate creation of a new Red Packet cell.
fn validate_creation() -> Result<(), i8> {
    // Validate Type ID (first 32 bytes of args)
    check_type_id(0, 32).map_err(|_| ERROR_TYPE_ID)?;

    // Load and parse output cell data
    let data = load_cell_data(0, Source::GroupOutput).map_err(|_| ERROR_INVALID_DATA)?;
    let rp = RedPacketData::from_bytes(&data).ok_or(ERROR_INVALID_DATA)?;

    // Validate initial state
    if rp.remaining_count != rp.total_count {
        return Err(ERROR_INVALID_INITIAL_STATE);
    }
    if rp.total_count == 0 {
        return Err(ERROR_INVALID_INITIAL_STATE);
    }
    if rp.total_amount == 0 {
        return Err(ERROR_INVALID_INITIAL_STATE);
    }
    if rp.mode != MODE_EQUAL && rp.mode != MODE_RANDOM {
        return Err(ERROR_INVALID_INITIAL_STATE);
    }
    if rp.expiry_since == 0 {
        return Err(ERROR_INVALID_INITIAL_STATE);
    }

    // Validate capacity
    let capacity =
        load_cell_capacity(0, Source::GroupOutput).map_err(|_| ERROR_CAPACITY_MISMATCH)?;
    if capacity
        < rp.total_amount
            .checked_add(MIN_CELL_CAPACITY)
            .ok_or(ERROR_CAPACITY_MISMATCH)?
    {
        return Err(ERROR_CAPACITY_MISMATCH);
    }

    // For equal mode, each share must be at least 1 shannon
    if rp.mode == MODE_EQUAL && rp.total_amount / (rp.total_count as u64) == 0 {
        return Err(ERROR_INVALID_AMOUNT);
    }

    Ok(())
}

/// Validate a claim operation (non-last share).
fn validate_claim() -> Result<(), i8> {
    let input_data_raw = load_cell_data(0, Source::GroupInput).map_err(|_| ERROR_INVALID_DATA)?;
    let output_data_raw =
        load_cell_data(0, Source::GroupOutput).map_err(|_| ERROR_INVALID_DATA)?;

    let input_data = RedPacketData::from_bytes(&input_data_raw).ok_or(ERROR_INVALID_DATA)?;
    let output_data = RedPacketData::from_bytes(&output_data_raw).ok_or(ERROR_INVALID_DATA)?;

    // Verify immutable fields
    if output_data.total_amount != input_data.total_amount
        || output_data.total_count != input_data.total_count
        || output_data.mode != input_data.mode
        || output_data.expiry_since != input_data.expiry_since
    {
        return Err(ERROR_IMMUTABLE_FIELD_CHANGED);
    }

    // Verify state transition
    if input_data.remaining_count <= 1 {
        return Err(ERROR_INVALID_STATE_TRANSITION);
    }
    if output_data.remaining_count != input_data.remaining_count - 1 {
        return Err(ERROR_INVALID_STATE_TRANSITION);
    }

    // Verify Lock and Type scripts unchanged
    let input_lock = load_cell_lock(0, Source::GroupInput).map_err(|_| ERROR_SCRIPT_MISMATCH)?;
    let output_lock = load_cell_lock(0, Source::GroupOutput).map_err(|_| ERROR_SCRIPT_MISMATCH)?;
    if input_lock.as_slice() != output_lock.as_slice() {
        return Err(ERROR_SCRIPT_MISMATCH);
    }
    let input_type = load_cell_type(0, Source::GroupInput).map_err(|_| ERROR_SCRIPT_MISMATCH)?;
    let output_type = load_cell_type(0, Source::GroupOutput).map_err(|_| ERROR_SCRIPT_MISMATCH)?;
    match (&input_type, &output_type) {
        (Some(a), Some(b)) if a.as_slice() == b.as_slice() => {}
        _ => return Err(ERROR_SCRIPT_MISMATCH),
    }

    // Extract claim info from witness
    let witness_args =
        load_witness_args(0, Source::GroupInput).map_err(|_| ERROR_INVALID_WITNESS)?;
    let lock_bytes: Vec<u8> = witness_args
        .lock()
        .to_opt()
        .ok_or(ERROR_INVALID_WITNESS)?
        .raw_data()
        .to_vec();
    let claim = ClaimAction::from_bytes(&lock_bytes).ok_or(ERROR_INVALID_WITNESS)?;

    // Validate amount
    validate_claim_amount(&input_data, claim.amount)?;

    // Verify capacity conservation
    let input_cap =
        load_cell_capacity(0, Source::GroupInput).map_err(|_| ERROR_CAPACITY_MISMATCH)?;
    let output_cap =
        load_cell_capacity(0, Source::GroupOutput).map_err(|_| ERROR_CAPACITY_MISMATCH)?;
    let expected_output_cap = input_cap
        .checked_sub(claim.amount)
        .ok_or(ERROR_CAPACITY_MISMATCH)?;
    if output_cap != expected_output_cap {
        return Err(ERROR_CAPACITY_MISMATCH);
    }
    if output_cap < MIN_CELL_CAPACITY {
        return Err(ERROR_CAPACITY_MISMATCH);
    }

    // Verify claimer output exists
    verify_claimer_output(&claim.claimer_lock_hash, claim.amount)?;

    Ok(())
}

/// Validate destruction (last claim or refund).
fn validate_destroy() -> Result<(), i8> {
    let input_data_raw = load_cell_data(0, Source::GroupInput).map_err(|_| ERROR_INVALID_DATA)?;
    let input_data = RedPacketData::from_bytes(&input_data_raw).ok_or(ERROR_INVALID_DATA)?;

    // Read witness to determine if this is a claim or refund
    let witness_args =
        load_witness_args(0, Source::GroupInput).map_err(|_| ERROR_INVALID_WITNESS)?;
    let lock_bytes: Vec<u8> = witness_args
        .lock()
        .to_opt()
        .ok_or(ERROR_INVALID_WITNESS)?
        .raw_data()
        .to_vec();

    let action_id = extract_action_id(&lock_bytes).ok_or(ERROR_INVALID_WITNESS)?;

    if action_id == ACTION_CLAIM {
        // Last share claim
        if input_data.remaining_count != 1 {
            return Err(ERROR_INVALID_DESTROY);
        }
        let claim = ClaimAction::from_bytes(&lock_bytes).ok_or(ERROR_INVALID_WITNESS)?;

        // For last share, validate amount > 0
        if claim.amount == 0 {
            return Err(ERROR_INVALID_AMOUNT);
        }

        // Verify claimer output exists
        verify_claimer_output(&claim.claimer_lock_hash, claim.amount)?;
    } else {
        // Refund — Lock Script handles since + signature validation.
        // Type Script only needs to verify remaining_count > 0.
        if input_data.remaining_count == 0 {
            return Err(ERROR_INVALID_DESTROY);
        }
    }

    Ok(())
}

/// Validate claim amount based on mode.
fn validate_claim_amount(data: &RedPacketData, amount: u64) -> Result<(), i8> {
    if amount == 0 {
        return Err(ERROR_INVALID_AMOUNT);
    }
    if data.mode == MODE_EQUAL {
        let per_share = data.total_amount / (data.total_count as u64);
        if amount != per_share {
            return Err(ERROR_INVALID_AMOUNT);
        }
    }
    // For random mode, just check amount > 0 (already checked above)
    Ok(())
}

/// Verify that an output cell with the claimer's lock hash exists
/// and has sufficient capacity.
fn verify_claimer_output(claimer_lock_hash: &[u8; 32], min_amount: u64) -> Result<(), i8> {
    let found = QueryIter::new(load_cell_lock_hash, Source::Output)
        .enumerate()
        .any(|(i, lock_hash)| {
            if lock_hash == *claimer_lock_hash {
                if let Ok(cap) = load_cell_capacity(i, Source::Output) {
                    return cap >= min_amount;
                }
            }
            false
        });

    if !found {
        return Err(ERROR_CLAIMER_OUTPUT_NOT_FOUND);
    }
    Ok(())
}
