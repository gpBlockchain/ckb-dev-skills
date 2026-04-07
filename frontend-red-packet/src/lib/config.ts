/**
 * Red Packet contract configuration loaded from environment variables.
 * Fill .env with actual deployment info after deploying contracts.
 */

export const config = {
  network: (import.meta.env.VITE_CKB_NETWORK as string) || "testnet",

  redPacketType: {
    codeHash:
      (import.meta.env.VITE_RED_PACKET_TYPE_CODE_HASH as string) ||
      "0x0000000000000000000000000000000000000000000000000000000000000000",
    hashType:
      (import.meta.env.VITE_RED_PACKET_TYPE_HASH_TYPE as string) || "type",
  },

  redPacketLock: {
    codeHash:
      (import.meta.env.VITE_RED_PACKET_LOCK_CODE_HASH as string) ||
      "0x0000000000000000000000000000000000000000000000000000000000000000",
    hashType:
      (import.meta.env.VITE_RED_PACKET_LOCK_HASH_TYPE as string) || "type",
  },

  typeCellDep: {
    txHash:
      (import.meta.env.VITE_RED_PACKET_TYPE_TX_HASH as string) ||
      "0x0000000000000000000000000000000000000000000000000000000000000000",
    index: Number(import.meta.env.VITE_RED_PACKET_TYPE_TX_INDEX ?? 0),
  },

  lockCellDep: {
    txHash:
      (import.meta.env.VITE_RED_PACKET_LOCK_TX_HASH as string) ||
      "0x0000000000000000000000000000000000000000000000000000000000000000",
    index: Number(import.meta.env.VITE_RED_PACKET_LOCK_TX_INDEX ?? 0),
  },
};

/** Returns true if contracts have been configured (not all zeros). */
export function isConfigured(): boolean {
  const zeros =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
  return (
    config.redPacketType.codeHash !== zeros &&
    config.redPacketLock.codeHash !== zeros
  );
}
