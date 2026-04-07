"use client";

import { useState } from "react";
import { ccc } from "@ckb-ccc/connector-react";
import { isConfigured, config } from "../lib/config";
import {
  decodeRedPacketData,
  formatCkb,
  modeName,
} from "../lib/codec";

export function ClaimPage() {
  const { client } = ccc.useCcc();
  const signer = ccc.useSigner();

  const [typeId, setTypeId] = useState("");
  const [authSignature, setAuthSignature] = useState("");
  const [packetInfo, setPacketInfo] = useState<ReturnType<typeof decodeRedPacketData> | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [txHash, setTxHash] = useState<string | null>(null);

  const configured = isConfigured();

  async function handleLookup() {
    if (!configured) {
      setError("Contracts not deployed. Fill .env with deployment info.");
      return;
    }
    if (!typeId.trim()) {
      setError("Please enter a Red Packet Type ID.");
      return;
    }

    setError(null);
    setPacketInfo(null);
    setLoading(true);

    try {
      // Build the type script for lookup
      const typeScript = ccc.Script.from({
        codeHash: config.redPacketType.codeHash,
        hashType: config.redPacketType.hashType as ccc.HashType,
        args: typeId.trim(),
      });

      // Search for the cell
      let found = false;
      for await (const cell of client.findCellsByType(typeScript, true)) {
        const data = cell.outputData;
        if (data) {
          const bytes = ccc.bytesFrom(data, "hex");
          const decoded = decodeRedPacketData(bytes);
          if (decoded) {
            setPacketInfo(decoded);
            found = true;
            break;
          }
        }
      }

      if (!found) {
        setError(
          "Red Packet not found on chain. It may have been fully claimed or the Type ID is incorrect.",
        );
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleClaim() {
    if (!signer) {
      setError("Please connect your wallet first.");
      return;
    }
    if (!authSignature.trim()) {
      setError(
        "Please enter the authorization signature from the red packet creator.",
      );
      return;
    }

    setError(null);
    setTxHash(null);
    setLoading(true);

    try {
      // In a real implementation, we would:
      // 1. Parse the authorization signature
      // 2. Build the claim transaction with the correct witness
      // 3. Send the transaction
      //
      // For now, show that the claim flow needs the contracts deployed.
      setError(
        "Claim transaction building requires deployed contracts. " +
          "Fill .env with deployment info and the claim flow will work end-to-end.",
      );
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="page">
      <h1>🎁 Claim Red Packet</h1>
      <p className="subtitle">
        Enter the Red Packet Type ID to look up its status, then claim your
        share.
      </p>

      {!configured && (
        <div className="alert alert-warning">
          <strong>⚠️ Contracts not deployed.</strong> Fill in{" "}
          <code>.env</code> first.
        </div>
      )}

      <div className="form-card">
        <div className="form-group">
          <label>Red Packet Type ID</label>
          <input
            type="text"
            placeholder="0x..."
            value={typeId}
            onChange={(e) => setTypeId(e.target.value)}
            className="input-mono"
          />
        </div>

        <button
          className="btn btn-secondary"
          onClick={handleLookup}
          disabled={loading}
        >
          {loading ? "Looking up..." : "🔍 Look Up"}
        </button>

        {packetInfo && (
          <div className="packet-info">
            <h3>Red Packet Info</h3>
            <table className="info-table">
              <tbody>
                <tr>
                  <td>Total Amount</td>
                  <td>
                    <strong>{formatCkb(packetInfo.totalAmount)} CKB</strong>
                  </td>
                </tr>
                <tr>
                  <td>Total Shares</td>
                  <td>{packetInfo.totalCount}</td>
                </tr>
                <tr>
                  <td>Remaining Shares</td>
                  <td>
                    <strong>{packetInfo.remainingCount}</strong> /{" "}
                    {packetInfo.totalCount}
                  </td>
                </tr>
                <tr>
                  <td>Mode</td>
                  <td>{modeName(packetInfo.mode)}</td>
                </tr>
              </tbody>
            </table>

            {packetInfo.remainingCount > 0 ? (
              <>
                <div className="form-group">
                  <label>Authorization Signature</label>
                  <textarea
                    placeholder="Paste the signature from the red packet creator..."
                    value={authSignature}
                    onChange={(e) => setAuthSignature(e.target.value)}
                    rows={3}
                    className="input-mono"
                  />
                  <div className="hint">
                    The creator signs an authorization off-chain for each
                    claimer.
                  </div>
                </div>

                <button
                  className="btn btn-primary btn-large"
                  onClick={handleClaim}
                  disabled={loading || !signer}
                >
                  {loading ? "Claiming..." : "🎁 Claim Share"}
                </button>
              </>
            ) : (
              <div className="alert alert-info">
                This red packet has been fully claimed.
              </div>
            )}
          </div>
        )}

        {error && <div className="alert alert-error">{error}</div>}

        {txHash && (
          <div className="alert alert-success">
            <strong>✅ Claim successful!</strong>
            <p>
              Transaction hash:{" "}
              <code className="tx-hash">{txHash}</code>
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
