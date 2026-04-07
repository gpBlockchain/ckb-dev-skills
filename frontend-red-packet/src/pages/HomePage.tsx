import { Link } from "react-router-dom";
import { isConfigured } from "../lib/config";

export function HomePage() {
  const configured = isConfigured();

  return (
    <div className="page">
      <div className="hero">
        <h1>🧧 CKB Red Packet</h1>
        <p className="subtitle">
          Decentralized red packets on the CKB blockchain.
          <br />
          Create, share, and claim CKB red packets with friends.
        </p>

        {!configured && (
          <div className="alert alert-warning">
            <strong>⚠️ Contracts not deployed yet.</strong>
            <p>
              Fill in <code>.env</code> with contract deployment information to
              enable full functionality. See <code>.env.example</code> for the
              required fields.
            </p>
          </div>
        )}

        <div className="card-grid">
          <Link to="/create" className="card card-action">
            <div className="card-icon">🧧</div>
            <h3>Create Red Packet</h3>
            <p>
              Create a new red packet and fund it with CKB. Choose equal or
              random distribution mode.
            </p>
          </Link>

          <Link to="/claim" className="card card-action">
            <div className="card-icon">🎁</div>
            <h3>Claim Red Packet</h3>
            <p>
              Enter a red packet ID to claim your share. Requires creator
              authorization.
            </p>
          </Link>

          <Link to="/my-packets" className="card card-action">
            <div className="card-icon">📋</div>
            <h3>My Red Packets</h3>
            <p>
              View red packets you have created. Refund expired ones to reclaim
              remaining CKB.
            </p>
          </Link>
        </div>
      </div>

      <div className="info-section">
        <h2>How It Works</h2>
        <div className="steps">
          <div className="step">
            <span className="step-num">1</span>
            <h4>Create</h4>
            <p>
              The creator deposits CKB into a red packet cell, choosing how many
              shares and the distribution mode (equal or random).
            </p>
          </div>
          <div className="step">
            <span className="step-num">2</span>
            <h4>Authorize</h4>
            <p>
              The creator signs authorization messages off-chain for each
              claimer, specifying their address and amount.
            </p>
          </div>
          <div className="step">
            <span className="step-num">3</span>
            <h4>Claim</h4>
            <p>
              Claimers submit the authorization signature to claim their share.
              The on-chain contract verifies everything.
            </p>
          </div>
          <div className="step">
            <span className="step-num">4</span>
            <h4>Refund</h4>
            <p>
              After expiry, the creator can refund any unclaimed CKB back to
              their wallet.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
