"use client";

import { ccc } from "@ckb-ccc/connector-react";

export function WalletButton() {
  const { open, wallet, disconnect } = ccc.useCcc();
  const signer = ccc.useSigner();

  if (wallet && signer) {
    return (
      <div className="wallet-connected">
        <span className="wallet-name">{wallet.name}</span>
        <button className="btn btn-sm" onClick={disconnect}>
          Disconnect
        </button>
      </div>
    );
  }

  return (
    <button className="btn btn-primary" onClick={open}>
      🔗 Connect Wallet
    </button>
  );
}
