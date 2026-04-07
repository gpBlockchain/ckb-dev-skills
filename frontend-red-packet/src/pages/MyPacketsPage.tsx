"use client";

import { useState, useEffect } from "react";
import { ccc } from "@ckb-ccc/connector-react";
import { isConfigured, config } from "../lib/config";
import {
  decodeRedPacketData,
  formatCkb,
  modeName,
} from "../lib/codec";
import type { RedPacketData } from "../lib/codec";

interface RedPacketCell {
  outPoint: { txHash: string; index: number };
  capacity: bigint;
  data: RedPacketData;
  typeId: string;
}

export function MyPacketsPage() {
  const { client } = ccc.useCcc();
  const signer = ccc.useSigner();

  const [packets, setPackets] = useState<RedPacketCell[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const configured = isConfigured();

  useEffect(() => {
    if (!signer || !configured) return;
    loadMyPackets();
  }, [signer, configured]);

  async function loadMyPackets() {
    if (!signer) return;
    setLoading(true);
    setError(null);

    try {
      const creatorLock = (await signer.getRecommendedAddressObj()).script;
      const creatorLockHash = creatorLock.hash();
      const creatorPubkeyHash = "0x" + creatorLockHash.slice(2, 42);

      // Build the lock script used for red packets
      const rpLock = ccc.Script.from({
        codeHash: config.redPacketLock.codeHash,
        hashType: config.redPacketLock.hashType as ccc.HashType,
        args: creatorPubkeyHash,
      });

      const rpType = ccc.Script.from({
        codeHash: config.redPacketType.codeHash,
        hashType: config.redPacketType.hashType as ccc.HashType,
        args: "0x",
      });

      // Search for cells with our lock and type scripts
      const found: RedPacketCell[] = [];
      for await (const cell of client.findCellsByLock(rpLock, rpType, true)) {
        const data = cell.outputData;
        if (data) {
          const bytes = ccc.bytesFrom(data, "hex");
          const decoded = decodeRedPacketData(bytes);
          if (decoded) {
            const typeScript = cell.cellOutput.type;
            const typeId = typeScript ? typeScript.args : "unknown";
            found.push({
              outPoint: {
                txHash: cell.outPoint.txHash,
                index: Number(cell.outPoint.index),
              },
              capacity: cell.cellOutput.capacity,
              data: decoded,
              typeId,
            });
          }
        }
      }

      setPackets(found);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleRefund(packet: RedPacketCell) {
    if (!signer) return;
    setError(null);

    try {
      // In a real implementation, we would:
      // 1. Build refund transaction with since >= expiry_since
      // 2. Sign with creator's key
      // 3. Send transaction
      setError(
        "Refund transaction building requires deployed contracts. " +
          "Fill .env with deployment info to enable refund.",
      );
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <div className="page">
      <h1>📋 My Red Packets</h1>
      <p className="subtitle">
        View red packets you have created and manage their status.
      </p>

      {!configured && (
        <div className="alert alert-warning">
          <strong>⚠️ Contracts not deployed.</strong> Fill in{" "}
          <code>.env</code> first.
        </div>
      )}

      {!signer && (
        <div className="alert alert-info">
          Connect your wallet to see your red packets.
        </div>
      )}

      {loading && <div className="loading">Loading your red packets...</div>}

      {error && <div className="alert alert-error">{error}</div>}

      {signer && configured && !loading && packets.length === 0 && (
        <div className="alert alert-info">
          No red packets found. Create one to get started!
        </div>
      )}

      {packets.length > 0 && (
        <div className="packets-list">
          {packets.map((pkt, i) => (
            <div key={i} className="card packet-card">
              <div className="packet-header">
                <span className="packet-mode-badge">
                  {modeName(pkt.data.mode)}
                </span>
                <span className="packet-status">
                  {pkt.data.remainingCount === 0
                    ? "✅ Fully Claimed"
                    : `${pkt.data.remainingCount}/${pkt.data.totalCount} remaining`}
                </span>
              </div>

              <div className="packet-amount">
                {formatCkb(pkt.data.totalAmount)} CKB
              </div>

              <table className="info-table">
                <tbody>
                  <tr>
                    <td>Type ID</td>
                    <td>
                      <code className="tx-hash">{pkt.typeId}</code>
                    </td>
                  </tr>
                  <tr>
                    <td>Shares</td>
                    <td>
                      {pkt.data.totalCount - pkt.data.remainingCount} claimed /{" "}
                      {pkt.data.totalCount} total
                    </td>
                  </tr>
                  <tr>
                    <td>Cell Capacity</td>
                    <td>{formatCkb(pkt.capacity)} CKB</td>
                  </tr>
                </tbody>
              </table>

              {pkt.data.remainingCount > 0 && (
                <button
                  className="btn btn-warning"
                  onClick={() => handleRefund(pkt)}
                >
                  🔙 Refund Remaining
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      {signer && configured && (
        <button
          className="btn btn-secondary"
          onClick={loadMyPackets}
          disabled={loading}
          style={{ marginTop: "1rem" }}
        >
          🔄 Refresh
        </button>
      )}
    </div>
  );
}
