"use client";

import { ccc } from "@ckb-ccc/connector-react";
import { useMemo, useState } from "react";
import type { ReactNode } from "react";
import { config } from "../lib/config";

export function CkbProvider({ children }: { children: ReactNode }) {
  const [network] = useState(config.network);

  const defaultClient = useMemo(() => {
    return network === "mainnet"
      ? new ccc.ClientPublicMainnet()
      : new ccc.ClientPublicTestnet();
  }, [network]);

  return (
    <ccc.Provider defaultClient={defaultClient}>
      {children}
    </ccc.Provider>
  );
}
