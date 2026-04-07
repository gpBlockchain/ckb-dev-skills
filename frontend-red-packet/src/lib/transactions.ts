/**
 * Transaction builders for Red Packet operations.
 *
 * Uses CCC SDK to construct CKB transactions for:
 * - Creating a new Red Packet
 * - Claiming a share from a Red Packet
 * - Refunding an expired Red Packet
 */

import type { ccc as cccTypes } from "@ckb-ccc/connector-react";
import { config } from "./config";
import {
  encodeRedPacketData,
  MIN_CELL_CAPACITY,
  MODE_EQUAL,
  MODE_RANDOM,
} from "./codec";

type Ccc = typeof cccTypes;

/** Helper: build the Red Packet Type Script with a given type_id as args. */
export function buildRedPacketTypeScript(
  ccc: Ccc,
  typeId: string,
): cccTypes.Script {
  return ccc.Script.from({
    codeHash: config.redPacketType.codeHash,
    hashType: config.redPacketType.hashType as cccTypes.HashType,
    args: typeId,
  });
}

/** Helper: build the Red Packet Lock Script with creator's pubkey hash as args. */
export function buildRedPacketLockScript(
  ccc: Ccc,
  creatorPubkeyHash: string,
): cccTypes.Script {
  return ccc.Script.from({
    codeHash: config.redPacketLock.codeHash,
    hashType: config.redPacketLock.hashType as cccTypes.HashType,
    args: creatorPubkeyHash,
  });
}

/** Build CellDeps for the Red Packet scripts. */
export function buildRedPacketCellDeps(ccc: Ccc): cccTypes.CellDep[] {
  return [
    ccc.CellDep.from({
      outPoint: {
        txHash: config.typeCellDep.txHash,
        index: config.typeCellDep.index,
      },
      depType: "code",
    }),
    ccc.CellDep.from({
      outPoint: {
        txHash: config.lockCellDep.txHash,
        index: config.lockCellDep.index,
      },
      depType: "code",
    }),
  ];
}

export interface CreateRedPacketParams {
  /** Total CKB to distribute, in shannons */
  totalAmount: bigint;
  /** Number of shares (1–255) */
  totalCount: number;
  /** Distribution mode: 0 = equal, 1 = random */
  mode: number;
  /** Expiry since value (CKB since format) */
  expirySince: bigint;
}

/**
 * Build a transaction that creates a new Red Packet cell.
 *
 * The caller (signer) is the creator. The Lock Script args will contain
 * the creator's blake160 pubkey hash.
 *
 * NOTE: The type_id is computed on-chain via the Type ID mechanism.
 * We use a placeholder (32 zero bytes) for args during initial TX building.
 * The actual type_id will be derived from the first input's OutPoint.
 */
export async function buildCreateRedPacketTx(
  ccc: Ccc,
  signer: cccTypes.Signer,
  params: CreateRedPacketParams,
): Promise<cccTypes.Transaction> {
  const { totalAmount, totalCount, mode, expirySince } = params;

  if (totalCount < 1 || totalCount > 255) {
    throw new Error("totalCount must be between 1 and 255");
  }
  if (mode !== MODE_EQUAL && mode !== MODE_RANDOM) {
    throw new Error("mode must be 0 (equal) or 1 (random)");
  }
  if (totalAmount <= 0n) {
    throw new Error("totalAmount must be positive");
  }

  // Get creator's lock script hash (blake160 of pubkey)
  const addresses = await signer.getAddresses();
  if (addresses.length === 0) throw new Error("No address available");
  const creatorLock = (await signer.getRecommendedAddressObj()).script;
  const creatorLockHash = creatorLock.hash();
  // blake160 = first 20 bytes of blake2b hash
  const creatorPubkeyHash = "0x" + creatorLockHash.slice(2, 42);

  // Build the Lock Script for the Red Packet cell
  const rpLock = buildRedPacketLockScript(ccc, creatorPubkeyHash);

  // Type Script with placeholder args (type_id will be set after knowing inputs)
  const typeIdPlaceholder =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
  const rpType = buildRedPacketTypeScript(ccc, typeIdPlaceholder);

  // Encode cell data
  const cellData = encodeRedPacketData({
    totalAmount,
    totalCount,
    remainingCount: totalCount,
    mode,
    expirySince,
  });

  // Capacity needed: totalAmount + minimum cell capacity
  const capacity = totalAmount + MIN_CELL_CAPACITY;

  // Build transaction
  const tx = ccc.Transaction.from({
    outputs: [
      {
        lock: rpLock,
        type: rpType,
        capacity: capacity,
      },
    ],
    outputsData: [ccc.bytesFrom(cellData)],
  });

  // Add cell deps for our scripts
  for (const dep of buildRedPacketCellDeps(ccc)) {
    tx.addCellDeps(dep);
  }

  // Auto-fill inputs from the signer's cells
  await tx.completeInputsByCapacity(signer);
  await tx.completeFeeBy(signer);

  return tx;
}

/**
 * Build the claim witness data (109 bytes).
 *
 * Layout:
 * | 0-3   | item_id (0u32 LE) |
 * | 4-68  | signature (65 bytes) |
 * | 69-100| claimer_lock_hash (32 bytes) |
 * | 101-108| amount (u64 LE) |
 */
export function buildClaimWitness(
  signature: Uint8Array,
  claimerLockHash: Uint8Array,
  amount: bigint,
): Uint8Array {
  const buf = new Uint8Array(109);
  const view = new DataView(buf.buffer);
  // item_id = 0 (claim)
  view.setUint32(0, 0, true);
  buf.set(signature.slice(0, 65), 4);
  buf.set(claimerLockHash.slice(0, 32), 69);
  view.setBigUint64(101, amount, true);
  return buf;
}

/**
 * Build the refund witness data (69 bytes).
 *
 * Layout:
 * | 0-3   | item_id (1u32 LE) |
 * | 4-68  | signature (65 bytes) |
 */
export function buildRefundWitness(signature: Uint8Array): Uint8Array {
  const buf = new Uint8Array(69);
  const view = new DataView(buf.buffer);
  // item_id = 1 (refund)
  view.setUint32(0, 1, true);
  buf.set(signature.slice(0, 65), 4);
  return buf;
}
