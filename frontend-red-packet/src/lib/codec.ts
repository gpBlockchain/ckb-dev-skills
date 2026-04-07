/**
 * Red Packet cell data encoding/decoding.
 *
 * Mirrors the Rust `RedPacketCellData` layout (19 bytes):
 * | Offset | Size | Field            |
 * |--------|------|------------------|
 * | 0      | 8    | total_amount     |
 * | 8      | 1    | total_count      |
 * | 9      | 1    | remaining_count  |
 * | 10     | 1    | mode             |
 * | 11     | 8    | expiry_since     |
 */

export const MODE_EQUAL = 0;
export const MODE_RANDOM = 1;
export const RED_PACKET_DATA_SIZE = 19;

/**
 * Minimum cell capacity for a red packet cell in shannons.
 * 91 CKB = 9_100_000_000 shannons
 */
export const MIN_CELL_CAPACITY = 9_100_000_000n;

/** 1 CKB = 10^8 shannons */
export const CKB_UNIT = 100_000_000n;

export interface RedPacketData {
  totalAmount: bigint; // in shannons
  totalCount: number; // 1–255
  remainingCount: number;
  mode: number; // 0 = equal, 1 = random
  expirySince: bigint; // CKB since value
}

/** Encode RedPacketData to 19-byte Uint8Array (little-endian). */
export function encodeRedPacketData(data: RedPacketData): Uint8Array {
  const buf = new Uint8Array(RED_PACKET_DATA_SIZE);
  const view = new DataView(buf.buffer);
  view.setBigUint64(0, data.totalAmount, true);
  buf[8] = data.totalCount;
  buf[9] = data.remainingCount;
  buf[10] = data.mode;
  view.setBigUint64(11, data.expirySince, true);
  return buf;
}

/** Decode RedPacketData from 19-byte Uint8Array. */
export function decodeRedPacketData(
  bytes: Uint8Array,
): RedPacketData | null {
  if (bytes.length !== RED_PACKET_DATA_SIZE) return null;
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  return {
    totalAmount: view.getBigUint64(0, true),
    totalCount: bytes[8],
    remainingCount: bytes[9],
    mode: bytes[10],
    expirySince: view.getBigUint64(11, true),
  };
}

/** Display amount in CKB (from shannons). */
export function formatCkb(shannons: bigint): string {
  const whole = shannons / CKB_UNIT;
  const frac = shannons % CKB_UNIT;
  if (frac === 0n) return `${whole}`;
  const fracStr = frac.toString().padStart(8, "0").replace(/0+$/, "");
  return `${whole}.${fracStr}`;
}

/** Parse a CKB amount string to shannons. */
export function parseCkb(ckb: string): bigint {
  const parts = ckb.trim().split(".");
  const whole = BigInt(parts[0] || "0") * CKB_UNIT;
  if (parts.length === 1) return whole;
  const fracStr = (parts[1] || "0").padEnd(8, "0").slice(0, 8);
  return whole + BigInt(fracStr);
}

/** Human-readable mode name. */
export function modeName(mode: number): string {
  return mode === MODE_EQUAL ? "Equal" : "Random";
}
