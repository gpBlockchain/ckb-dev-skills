"use client";

import { Link, useLocation } from "react-router-dom";
import { WalletButton } from "./WalletButton";

export function Header() {
  const location = useLocation();

  const links = [
    { to: "/", label: "🏠 Home" },
    { to: "/create", label: "🧧 Create" },
    { to: "/claim", label: "🎁 Claim" },
    { to: "/my-packets", label: "📋 My Packets" },
  ];

  return (
    <header className="header">
      <div className="header-inner">
        <Link to="/" className="logo">
          🧧 CKB Red Packet
        </Link>
        <nav className="nav">
          {links.map((link) => (
            <Link
              key={link.to}
              to={link.to}
              className={`nav-link ${location.pathname === link.to ? "active" : ""}`}
            >
              {link.label}
            </Link>
          ))}
        </nav>
        <WalletButton />
      </div>
    </header>
  );
}
