use ckb_crypto::secp::{Generator, Privkey};
use ckb_hash::{blake2b_256, new_blake2b};
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::TransactionBuilder,
    packed::*,
    prelude::*,
};
use ckb_testtool::context::Context;

use red_packet_common::{
    ClaimAction, RedPacketData, RefundAction, CLAIM_DOMAIN, MIN_CELL_CAPACITY, MODE_EQUAL,
    MODE_RANDOM,
};

const MAX_CYCLES: u64 = 100_000_000;
const ONE_CKB: u64 = 100_000_000; // 1 CKB = 10^8 shannons

// ─── Helpers ────────────────────────────────────────────────────────────────

fn blake160(data: &[u8]) -> [u8; 20] {
    let mut buf = [0u8; 20];
    let hash = blake2b_256(data);
    buf.clone_from_slice(&hash[..20]);
    buf
}

/// Pack a u64 as Uint64 for capacity.
fn pack_capacity(v: u64) -> Uint64 {
    v.pack()
}

/// Pack a u64 as since value.
fn pack_since(v: u64) -> Uint64 {
    v.pack()
}

/// Build a claim authorization message:
/// blake2b("RED_PACKET_CLAIM" || claimer_lock_hash || amount || type_id)
fn build_claim_message(claimer_lock_hash: &[u8; 32], amount: u64, type_id: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut blake2b = new_blake2b();
    blake2b.update(CLAIM_DOMAIN);
    blake2b.update(claimer_lock_hash);
    blake2b.update(&amount.to_le_bytes());
    blake2b.update(type_id);
    blake2b.finalize(&mut result);
    result
}

/// Sign a message with a private key, returning a 65-byte signature
/// in the format: r (32) || s (32) || recovery_id (1)
fn sign_message(key: &Privkey, message: &[u8; 32]) -> [u8; 65] {
    use ckb_testtool::ckb_types::H256;
    let msg = H256::from(*message);
    let sig = key.sign_recoverable(&msg).expect("sign");
    let mut result = [0u8; 65];
    result.copy_from_slice(&sig.serialize());
    result
}

/// Build script hash for a lock script (blake2b of the entire script).
fn compute_script_hash(script: &Script) -> [u8; 32] {
    blake2b_256(script.as_slice())
}

/// Deploy contracts and create a basic context.
struct TestSetup {
    context: Context,
    type_out_point: OutPoint,
    lock_out_point: OutPoint,
    always_success_out_point: OutPoint,
    creator_privkey: Privkey,
    creator_pubkey_hash: [u8; 20],
}

impl TestSetup {
    fn new() -> Self {
        use ckb_testtool::builtin::ALWAYS_SUCCESS;

        let mut context = Context::default();

        // Deploy our contracts
        let type_out_point = context.deploy_cell_by_name("red-packet-type");
        let lock_out_point = context.deploy_cell_by_name("red-packet-lock");

        // Deploy always_success for use as a neutral lock in Type Script tests
        let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

        // Generate creator key pair
        let creator_privkey = Generator::random_privkey();
        let creator_pubkey = creator_privkey.pubkey().expect("pubkey");
        let creator_pubkey_hash = blake160(&creator_pubkey.serialize());

        Self {
            context,
            type_out_point,
            lock_out_point,
            always_success_out_point,
            creator_privkey,
            creator_pubkey_hash,
        }
    }

    /// Build a simple always-success lock (for Type Script tests, avoids Lock Script interference)
    fn build_always_success_lock(&mut self, args: &[u8]) -> Script {
        self.context
            .build_script(
                &self.always_success_out_point,
                Bytes::from(args.to_vec()),
            )
            .expect("always_success lock")
    }

    /// Build the red packet lock script
    fn build_rp_lock(&mut self) -> Script {
        self.context
            .build_script(
                &self.lock_out_point,
                Bytes::from(self.creator_pubkey_hash.to_vec()),
            )
            .expect("lock script")
    }

    /// Build the red packet type script with specific args (type_id)
    fn build_rp_type(&mut self, args: Bytes) -> Script {
        self.context
            .build_script(&self.type_out_point, args)
            .expect("type script")
    }

    /// Build a simple always-success lock for the claimer
    fn build_claimer_lock(&mut self, claimer_id: &[u8]) -> Script {
        self.build_always_success_lock(claimer_id)
    }
}

// ─── Type Script Tests ──────────────────────────────────────────────────────

/// Test: Create a valid red packet (equal mode)
#[test]
fn test_create_red_packet_equal_mode() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    // Create a funding input cell
    let funding_capacity = 200 * ONE_CKB;
    let input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(funding_capacity))
            .lock(rp_lock.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();

    // Red packet data: 100 CKB total, 5 shares, equal mode
    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 5,
        remaining_count: 5,
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001, // absolute epoch 1
    };

    // Compute Type ID: blake2b(first_input_cellinput.as_slice() || output_index)
    // The CellInput includes both the OutPoint AND the since field
    let input_for_type_id = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();
    let mut type_id_hasher = new_blake2b();
    type_id_hasher.update(input_for_type_id.as_slice());
    type_id_hasher.update(&0u64.to_le_bytes());
    let mut type_id = [0u8; 32];
    type_id_hasher.finalize(&mut type_id);

    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = rp_data.total_amount + MIN_CELL_CAPACITY;

    let outputs = vec![
        // Red packet cell
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity))
            .lock(rp_lock.clone())
            .type_(Some(rp_type).pack())
            .build(),
        // Change cell
        CellOutput::new_builder()
            .capacity(pack_capacity(funding_capacity - rp_capacity))
            .lock(rp_lock)
            .build(),
    ];

    let outputs_data = vec![Bytes::from(rp_data.to_bytes().to_vec()), Bytes::new()];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let cycles = setup
        .context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("create red packet should pass");
    println!("Create red packet cycles: {}", cycles);
}

/// Test: Create a valid red packet (random mode)
#[test]
fn test_create_red_packet_random_mode() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    let funding_capacity = 200 * ONE_CKB;
    let input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(funding_capacity))
            .lock(rp_lock.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();

    let rp_data = RedPacketData {
        total_amount: 50 * ONE_CKB,
        total_count: 3,
        remaining_count: 3,
        mode: MODE_RANDOM,
        expiry_since: 0x2000_0000_0000_0001,
    };

    let input_for_type_id = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();
    let mut type_id_hasher = new_blake2b();
    type_id_hasher.update(input_for_type_id.as_slice());
    type_id_hasher.update(&0u64.to_le_bytes());
    let mut type_id = [0u8; 32];
    type_id_hasher.finalize(&mut type_id);

    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = rp_data.total_amount + MIN_CELL_CAPACITY;

    let outputs = vec![CellOutput::new_builder()
        .capacity(pack_capacity(rp_capacity))
        .lock(rp_lock)
        .type_(Some(rp_type).pack())
        .build()];
    let outputs_data = vec![Bytes::from(rp_data.to_bytes().to_vec())];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    setup
        .context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("create random red packet should pass");
}

/// Test: Create fails with remaining_count != total_count
#[test]
fn test_create_fails_remaining_count_mismatch() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    let funding_capacity = 200 * ONE_CKB;
    let input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(funding_capacity))
            .lock(rp_lock.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();

    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 5,
        remaining_count: 4, // WRONG: should be 5
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001,
    };

    let input_for_type_id = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();
    let mut type_id_hasher = new_blake2b();
    type_id_hasher.update(input_for_type_id.as_slice());
    type_id_hasher.update(&0u64.to_le_bytes());
    let mut type_id = [0u8; 32];
    type_id_hasher.finalize(&mut type_id);

    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = rp_data.total_amount + MIN_CELL_CAPACITY;

    let outputs = vec![CellOutput::new_builder()
        .capacity(pack_capacity(rp_capacity))
        .lock(rp_lock)
        .type_(Some(rp_type).pack())
        .build()];
    let outputs_data = vec![Bytes::from(rp_data.to_bytes().to_vec())];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let result = setup.context.verify_tx(&tx, MAX_CYCLES);
    assert!(result.is_err(), "should fail: remaining_count != total_count");
}

/// Test: Create fails with zero total_count
#[test]
fn test_create_fails_zero_count() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    let funding_capacity = 200 * ONE_CKB;
    let input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(funding_capacity))
            .lock(rp_lock.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();

    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 0,
        remaining_count: 0,
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001,
    };

    let input_for_type_id = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();
    let mut type_id_hasher = new_blake2b();
    type_id_hasher.update(input_for_type_id.as_slice());
    type_id_hasher.update(&0u64.to_le_bytes());
    let mut type_id = [0u8; 32];
    type_id_hasher.finalize(&mut type_id);

    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));

    let outputs = vec![CellOutput::new_builder()
        .capacity(pack_capacity(funding_capacity))
        .lock(rp_lock)
        .type_(Some(rp_type).pack())
        .build()];
    let outputs_data = vec![Bytes::from(rp_data.to_bytes().to_vec())];

    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let result = setup.context.verify_tx(&tx, MAX_CYCLES);
    assert!(result.is_err(), "should fail: zero total_count");
}

/// Test: Claim a share (equal mode, non-last)
#[test]
fn test_claim_equal_mode() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    // Red packet state: 100 CKB, 5 shares, 5 remaining
    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 5,
        remaining_count: 5,
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001,
    };
    let type_id = [0x42u8; 32]; // arbitrary type_id for existing cell
    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = rp_data.total_amount + MIN_CELL_CAPACITY;

    // Create the existing red packet cell (as input)
    let rp_input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity))
            .lock(rp_lock.clone())
            .type_(Some(rp_type.clone()).pack())
            .build(),
        Bytes::from(rp_data.to_bytes().to_vec()),
    );
    let rp_input = CellInput::new_builder()
        .previous_output(rp_input_out_point)
        .build();

    // Claim amount (equal mode): 100 CKB / 5 = 20 CKB
    let claim_amount = 20 * ONE_CKB;

    // Claimer's lock script
    let claimer_lock = setup.build_claimer_lock(b"claimer1");
    let claimer_lock_hash = compute_script_hash(&claimer_lock);

    // Build claim authorization signature
    let claim_message = build_claim_message(&claimer_lock_hash, claim_amount, &type_id);
    let claim_signature = sign_message(&setup.creator_privkey, &claim_message);

    let claim_action = ClaimAction {
        signature: claim_signature,
        claimer_lock_hash,
        amount: claim_amount,
    };

    // Updated red packet data
    let output_rp_data = RedPacketData {
        remaining_count: 4,
        ..rp_data
    };

    // Build outputs
    let outputs = vec![
        // Updated red packet cell
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity - claim_amount))
            .lock(rp_lock.clone())
            .type_(Some(rp_type).pack())
            .build(),
        // Claimer's cell
        CellOutput::new_builder()
            .capacity(pack_capacity(claim_amount))
            .lock(claimer_lock)
            .build(),
    ];
    let outputs_data = vec![
        Bytes::from(output_rp_data.to_bytes().to_vec()),
        Bytes::new(),
    ];

    // Build witness with ClaimAction
    let witness_lock = Bytes::from(claim_action.to_bytes().to_vec());
    let witness = WitnessArgs::new_builder()
        .lock(Some(witness_lock).pack())
        .build();

    let tx = TransactionBuilder::default()
        .input(rp_input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness.as_bytes().pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let cycles = setup
        .context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("claim should pass");
    println!("Claim cycles: {}", cycles);
}

/// Test: Claim fails with wrong amount (equal mode)
#[test]
fn test_claim_fails_wrong_amount_equal_mode() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 5,
        remaining_count: 5,
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001,
    };
    let type_id = [0x42u8; 32];
    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = rp_data.total_amount + MIN_CELL_CAPACITY;

    let rp_input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity))
            .lock(rp_lock.clone())
            .type_(Some(rp_type.clone()).pack())
            .build(),
        Bytes::from(rp_data.to_bytes().to_vec()),
    );
    let rp_input = CellInput::new_builder()
        .previous_output(rp_input_out_point)
        .build();

    // Wrong claim amount: should be 20 CKB, trying 30 CKB
    let wrong_amount = 30 * ONE_CKB;

    let claimer_lock = setup.build_claimer_lock(b"claimer1");
    let claimer_lock_hash = compute_script_hash(&claimer_lock);
    let claim_message = build_claim_message(&claimer_lock_hash, wrong_amount, &type_id);
    let claim_signature = sign_message(&setup.creator_privkey, &claim_message);

    let claim_action = ClaimAction {
        signature: claim_signature,
        claimer_lock_hash,
        amount: wrong_amount,
    };

    let output_rp_data = RedPacketData {
        remaining_count: 4,
        ..rp_data
    };

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity - wrong_amount))
            .lock(rp_lock.clone())
            .type_(Some(rp_type).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(pack_capacity(wrong_amount))
            .lock(claimer_lock)
            .build(),
    ];
    let outputs_data = vec![
        Bytes::from(output_rp_data.to_bytes().to_vec()),
        Bytes::new(),
    ];

    let witness_lock = Bytes::from(claim_action.to_bytes().to_vec());
    let witness = WitnessArgs::new_builder()
        .lock(Some(witness_lock).pack())
        .build();

    let tx = TransactionBuilder::default()
        .input(rp_input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness.as_bytes().pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let result = setup.context.verify_tx(&tx, MAX_CYCLES);
    assert!(result.is_err(), "should fail: wrong claim amount");
}

/// Test: Claim fails when immutable field is changed
#[test]
fn test_claim_fails_immutable_field_changed() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 5,
        remaining_count: 5,
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001,
    };
    let type_id = [0x42u8; 32];
    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = rp_data.total_amount + MIN_CELL_CAPACITY;

    let rp_input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity))
            .lock(rp_lock.clone())
            .type_(Some(rp_type.clone()).pack())
            .build(),
        Bytes::from(rp_data.to_bytes().to_vec()),
    );
    let rp_input = CellInput::new_builder()
        .previous_output(rp_input_out_point)
        .build();

    let claim_amount = 20 * ONE_CKB;
    let claimer_lock = setup.build_claimer_lock(b"claimer1");
    let claimer_lock_hash = compute_script_hash(&claimer_lock);
    let claim_message = build_claim_message(&claimer_lock_hash, claim_amount, &type_id);
    let claim_signature = sign_message(&setup.creator_privkey, &claim_message);

    let claim_action = ClaimAction {
        signature: claim_signature,
        claimer_lock_hash,
        amount: claim_amount,
    };

    // Tamper with output data: change total_amount
    let output_rp_data = RedPacketData {
        total_amount: 50 * ONE_CKB, // CHANGED from 100
        remaining_count: 4,
        ..rp_data
    };

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity - claim_amount))
            .lock(rp_lock.clone())
            .type_(Some(rp_type).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(pack_capacity(claim_amount))
            .lock(claimer_lock)
            .build(),
    ];
    let outputs_data = vec![
        Bytes::from(output_rp_data.to_bytes().to_vec()),
        Bytes::new(),
    ];

    let witness_lock = Bytes::from(claim_action.to_bytes().to_vec());
    let witness = WitnessArgs::new_builder()
        .lock(Some(witness_lock).pack())
        .build();

    let tx = TransactionBuilder::default()
        .input(rp_input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness.as_bytes().pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let result = setup.context.verify_tx(&tx, MAX_CYCLES);
    assert!(
        result.is_err(),
        "should fail: immutable field total_amount changed"
    );
}

/// Test: Destroy via last claim
#[test]
fn test_destroy_last_claim() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    // Red packet with only 1 remaining share
    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 5,
        remaining_count: 1,
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001,
    };
    let type_id = [0x42u8; 32];
    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));

    // Last share gets the remaining: total_amount - 4 * (total_amount / 5) + MIN_CELL_CAPACITY
    // = 100 - 4*20 + 91 = 111 CKB, but capacity is total
    let remaining_capacity = rp_data.total_amount - 4 * (rp_data.total_amount / 5) + MIN_CELL_CAPACITY;

    let rp_input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(remaining_capacity))
            .lock(rp_lock.clone())
            .type_(Some(rp_type.clone()).pack())
            .build(),
        Bytes::from(rp_data.to_bytes().to_vec()),
    );
    let rp_input = CellInput::new_builder()
        .previous_output(rp_input_out_point)
        .build();

    let claimer_lock = setup.build_claimer_lock(b"last_claimer");
    let claimer_lock_hash = compute_script_hash(&claimer_lock);
    let claim_amount = remaining_capacity; // gets all remaining

    let claim_message = build_claim_message(&claimer_lock_hash, claim_amount, &type_id);
    let claim_signature = sign_message(&setup.creator_privkey, &claim_message);

    let claim_action = ClaimAction {
        signature: claim_signature,
        claimer_lock_hash,
        amount: claim_amount,
    };

    // No red packet output (cell destroyed)
    let outputs = vec![CellOutput::new_builder()
        .capacity(pack_capacity(claim_amount))
        .lock(claimer_lock)
        .build()];
    let outputs_data = vec![Bytes::new()];

    let witness_lock = Bytes::from(claim_action.to_bytes().to_vec());
    let witness = WitnessArgs::new_builder()
        .lock(Some(witness_lock).pack())
        .build();

    let tx = TransactionBuilder::default()
        .input(rp_input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness.as_bytes().pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let cycles = setup
        .context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("last claim should pass");
    println!("Last claim cycles: {}", cycles);
}

/// Test: Destroy via refund (expired red packet)
#[test]
fn test_destroy_refund_expired() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    // Red packet with 3 remaining shares, expired
    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 5,
        remaining_count: 3,
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001, // absolute epoch 1
    };
    let type_id = [0x42u8; 32];
    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = 60 * ONE_CKB + MIN_CELL_CAPACITY; // some CKB already claimed

    let rp_input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity))
            .lock(rp_lock.clone())
            .type_(Some(rp_type).pack())
            .build(),
        Bytes::from(rp_data.to_bytes().to_vec()),
    );

    // Set since >= expiry_since to simulate time passing
    let rp_input = CellInput::new_builder()
        .previous_output(rp_input_out_point)
        .since(pack_since(rp_data.expiry_since)) // since == expiry_since
        .build();

    // Refund: sign the tx hash
    // For the refund, we need to sign the transaction hash.
    // We build the tx first, then sign.

    // Creator gets all remaining CKB
    let creator_lock = setup.build_claimer_lock(b"creator");
    let outputs = vec![CellOutput::new_builder()
        .capacity(pack_capacity(rp_capacity))
        .lock(creator_lock)
        .build()];
    let outputs_data = vec![Bytes::new()];

    // First build with a placeholder witness to get the tx hash
    let refund_action_placeholder = RefundAction {
        signature: [0u8; 65],
    };
    let witness_lock_placeholder = Bytes::from(refund_action_placeholder.to_bytes().to_vec());
    let witness_placeholder = WitnessArgs::new_builder()
        .lock(Some(witness_lock_placeholder).pack())
        .build();

    let tx = TransactionBuilder::default()
        .input(rp_input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness_placeholder.as_bytes().pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    // Now sign the tx hash
    let tx_hash: [u8; 32] = tx.hash().into();
    let refund_signature = sign_message(&setup.creator_privkey, &tx_hash);

    let refund_action = RefundAction {
        signature: refund_signature,
    };
    let witness_lock = Bytes::from(refund_action.to_bytes().to_vec());
    let witness = WitnessArgs::new_builder()
        .lock(Some(witness_lock).pack())
        .build();

    // Rebuild with actual signature
    let tx = tx
        .as_advanced_builder()
        .set_witnesses(vec![witness.as_bytes().pack()])
        .build();

    let cycles = setup
        .context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("refund should pass");
    println!("Refund cycles: {}", cycles);
}

/// Test: Random mode claim with valid amount
#[test]
fn test_claim_random_mode() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 3,
        remaining_count: 3,
        mode: MODE_RANDOM,
        expiry_since: 0x2000_0000_0000_0001,
    };
    let type_id = [0x42u8; 32];
    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = rp_data.total_amount + MIN_CELL_CAPACITY;

    let rp_input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity))
            .lock(rp_lock.clone())
            .type_(Some(rp_type.clone()).pack())
            .build(),
        Bytes::from(rp_data.to_bytes().to_vec()),
    );
    let rp_input = CellInput::new_builder()
        .previous_output(rp_input_out_point)
        .build();

    // Random amount: 15 CKB (different from equal share of 33.33 CKB)
    let claim_amount = 15 * ONE_CKB;
    let claimer_lock = setup.build_claimer_lock(b"lucky_claimer");
    let claimer_lock_hash = compute_script_hash(&claimer_lock);
    let claim_message = build_claim_message(&claimer_lock_hash, claim_amount, &type_id);
    let claim_signature = sign_message(&setup.creator_privkey, &claim_message);

    let claim_action = ClaimAction {
        signature: claim_signature,
        claimer_lock_hash,
        amount: claim_amount,
    };

    let output_rp_data = RedPacketData {
        remaining_count: 2,
        ..rp_data
    };

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity - claim_amount))
            .lock(rp_lock.clone())
            .type_(Some(rp_type).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(pack_capacity(claim_amount))
            .lock(claimer_lock)
            .build(),
    ];
    let outputs_data = vec![
        Bytes::from(output_rp_data.to_bytes().to_vec()),
        Bytes::new(),
    ];

    let witness_lock = Bytes::from(claim_action.to_bytes().to_vec());
    let witness = WitnessArgs::new_builder()
        .lock(Some(witness_lock).pack())
        .build();

    let tx = TransactionBuilder::default()
        .input(rp_input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness.as_bytes().pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let cycles = setup
        .context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("random claim should pass");
    println!("Random claim cycles: {}", cycles);
}

/// Test: Claim fails when remaining_count not decremented properly
#[test]
fn test_claim_fails_wrong_remaining_count() {
    let mut setup = TestSetup::new();
    let rp_lock = setup.build_always_success_lock(b"creator");

    let rp_data = RedPacketData {
        total_amount: 100 * ONE_CKB,
        total_count: 5,
        remaining_count: 5,
        mode: MODE_EQUAL,
        expiry_since: 0x2000_0000_0000_0001,
    };
    let type_id = [0x42u8; 32];
    let rp_type = setup.build_rp_type(Bytes::from(type_id.to_vec()));
    let rp_capacity = rp_data.total_amount + MIN_CELL_CAPACITY;

    let rp_input_out_point = setup.context.create_cell(
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity))
            .lock(rp_lock.clone())
            .type_(Some(rp_type.clone()).pack())
            .build(),
        Bytes::from(rp_data.to_bytes().to_vec()),
    );
    let rp_input = CellInput::new_builder()
        .previous_output(rp_input_out_point)
        .build();

    let claim_amount = 20 * ONE_CKB;
    let claimer_lock = setup.build_claimer_lock(b"claimer1");
    let claimer_lock_hash = compute_script_hash(&claimer_lock);
    let claim_message = build_claim_message(&claimer_lock_hash, claim_amount, &type_id);
    let claim_signature = sign_message(&setup.creator_privkey, &claim_message);

    let claim_action = ClaimAction {
        signature: claim_signature,
        claimer_lock_hash,
        amount: claim_amount,
    };

    // Output data with wrong remaining_count (3 instead of 4)
    let output_rp_data = RedPacketData {
        remaining_count: 3, // WRONG: should be 4
        ..rp_data
    };

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(pack_capacity(rp_capacity - claim_amount))
            .lock(rp_lock.clone())
            .type_(Some(rp_type).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(pack_capacity(claim_amount))
            .lock(claimer_lock)
            .build(),
    ];
    let outputs_data = vec![
        Bytes::from(output_rp_data.to_bytes().to_vec()),
        Bytes::new(),
    ];

    let witness_lock = Bytes::from(claim_action.to_bytes().to_vec());
    let witness = WitnessArgs::new_builder()
        .lock(Some(witness_lock).pack())
        .build();

    let tx = TransactionBuilder::default()
        .input(rp_input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witness(witness.as_bytes().pack())
        .build();
    let tx = setup.context.complete_tx(tx);

    let result = setup.context.verify_tx(&tx, MAX_CYCLES);
    assert!(
        result.is_err(),
        "should fail: remaining_count not decremented correctly"
    );
}

// ─── Unit tests for common types ────────────────────────────────────────────

#[test]
fn test_red_packet_data_roundtrip() {
    let data = RedPacketData {
        total_amount: 12345678,
        total_count: 10,
        remaining_count: 7,
        mode: MODE_RANDOM,
        expiry_since: 0xDEADBEEF,
    };
    let bytes = data.to_bytes();
    let decoded = RedPacketData::from_bytes(&bytes).expect("decode");
    assert_eq!(decoded.total_amount, data.total_amount);
    assert_eq!(decoded.total_count, data.total_count);
    assert_eq!(decoded.remaining_count, data.remaining_count);
    assert_eq!(decoded.mode, data.mode);
    assert_eq!(decoded.expiry_since, data.expiry_since);
}

#[test]
fn test_claim_action_roundtrip() {
    let action = ClaimAction {
        signature: [0xAB; 65],
        claimer_lock_hash: [0xCD; 32],
        amount: 999999,
    };
    let bytes = action.to_bytes();
    let decoded = ClaimAction::from_bytes(&bytes).expect("decode");
    assert_eq!(decoded.signature, action.signature);
    assert_eq!(decoded.claimer_lock_hash, action.claimer_lock_hash);
    assert_eq!(decoded.amount, action.amount);
}

#[test]
fn test_refund_action_roundtrip() {
    let action = RefundAction {
        signature: [0xEF; 65],
    };
    let bytes = action.to_bytes();
    let decoded = RefundAction::from_bytes(&bytes).expect("decode");
    assert_eq!(decoded.signature, action.signature);
}

#[test]
fn test_invalid_data_size() {
    assert!(RedPacketData::from_bytes(&[0u8; 18]).is_none());
    assert!(RedPacketData::from_bytes(&[0u8; 20]).is_none());
    assert!(RedPacketData::from_bytes(&[]).is_none());
}
