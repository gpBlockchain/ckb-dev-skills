"use client";

import { useState } from "react";
import { ccc } from "@ckb-ccc/connector-react";
import { isConfigured } from "../lib/config";
import { buildCreateRedPacketTx } from "../lib/transactions";
import { MODE_EQUAL, MODE_RANDOM, parseCkb, CKB_UNIT } from "../lib/codec";

export function CreatePage() {
  const { client } = ccc.useCcc();
  const signer = ccc.useSigner();

  const [totalCkb, setTotalCkb] = useState("");
  const [totalCount, setTotalCount] = useState("5");
  const [mode, setMode] = useState<number>(MODE_EQUAL);
  const [expiryHours, setExpiryHours] = useState("24");
  const [txHash, setTxHash] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const configured = isConfigured();

  async function handleCreate() {
    if (!signer) {
      setError("Please connect your wallet first.");
      return;
    }
    if (!configured) {
      setError("Contracts not deployed. Fill .env with deployment info.");
      return;
    }

    setError(null);
    setTxHash(null);
    setLoading(true);

    try {
      const totalAmount = parseCkb(totalCkb);
      const count = parseInt(totalCount, 10);
      const hours = parseInt(expiryHours, 10);

      if (totalAmount < CKB_UNIT) {
        throw new Error("Total amount must be at least 1 CKB");
      }
      if (count < 1 || count > 255) {
        throw new Error("Number of shares must be between 1 and 255");
      }
      if (mode === MODE_EQUAL && totalAmount / BigInt(count) === 0n) {
        throw new Error("Each share must be at least 1 shannon");
      }

      // Build an epoch-based expiry since value
      // For simplicity, use absolute timestamp since (flag 0x40)
      // 0x4000000000000000 | timestamp_in_seconds
      const expiryTimestamp = BigInt(
        Math.floor(Date.now() / 1000) + hours * 3600,
      );
      const expirySince = 0x4000000000000000n | expiryTimestamp;

      const tx = await buildCreateRedPacketTx(ccc, signer, {
        totalAmount,
        totalCount: count,
        mode,
        expirySince,
      });

      const hash = await signer.sendTransaction(tx);
      setTxHash(hash);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  const perShare =
    totalCkb && totalCount
      ? (() => {
          try {
            const amt = parseCkb(totalCkb);
            const cnt = BigInt(totalCount);
            if (cnt <= 0n) return "—";
            const share = amt / cnt;
            const whole = share / CKB_UNIT;
            const frac = share % CKB_UNIT;
            if (frac === 0n) return `${whole} CKB`;
            const fracStr = frac.toString().padStart(8, "0").replace(/0+$/, "");
            return `${whole}.${fracStr} CKB`;
          } catch {
            return "—";
          }
        })()
      : "—";

  return (
    <div className="page">
      <h1>🧧 Create Red Packet</h1>
      <p className="subtitle">
        Fund a new red packet on the CKB blockchain for your friends to claim.
      </p>

      {!configured && (
        <div className="alert alert-warning">
          <strong>⚠️ Contracts not deployed.</strong> Fill in{" "}
          <code>.env</code> first.
        </div>
      )}

      <div className="form-card">
        <div className="form-group">
          <label>Total Amount (CKB)</label>
          <input
            type="text"
            placeholder="e.g. 100"
            value={totalCkb}
            onChange={(e) => setTotalCkb(e.target.value)}
          />
        </div>

        <div className="form-group">
          <label>Number of Shares</label>
          <input
            type="number"
            min={1}
            max={255}
            value={totalCount}
            onChange={(e) => setTotalCount(e.target.value)}
          />
        </div>

        <div className="form-group">
          <label>Distribution Mode</label>
          <div className="radio-group">
            <label className="radio-label">
              <input
                type="radio"
                name="mode"
                checked={mode === MODE_EQUAL}
                onChange={() => setMode(MODE_EQUAL)}
              />
              Equal — Each person gets the same amount
            </label>
            <label className="radio-label">
              <input
                type="radio"
                name="mode"
                checked={mode === MODE_RANDOM}
                onChange={() => setMode(MODE_RANDOM)}
              />
              Random — Random distribution (set by creator)
            </label>
          </div>
          {mode === MODE_EQUAL && (
            <div className="hint">
              Per share: <strong>{perShare}</strong>
            </div>
          )}
        </div>

        <div className="form-group">
          <label>Expiry (hours from now)</label>
          <input
            type="number"
            min={1}
            max={720}
            value={expiryHours}
            onChange={(e) => setExpiryHours(e.target.value)}
          />
          <div className="hint">
            Unclaimed CKB can be refunded after expiry.
          </div>
        </div>

        <button
          className="btn btn-primary btn-large"
          onClick={handleCreate}
          disabled={loading || !signer}
        >
          {loading ? "Creating..." : "🧧 Create Red Packet"}
        </button>

        {error && <div className="alert alert-error">{error}</div>}

        {txHash && (
          <div className="alert alert-success">
            <strong>✅ Red Packet created!</strong>
            <p>
              Transaction hash:{" "}
              <code className="tx-hash">{txHash}</code>
            </p>
            <p className="hint">
              Share this with your friends. They&apos;ll need the red packet ID
              (type_id) and your authorization signature to claim.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
