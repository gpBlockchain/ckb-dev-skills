"use client";

import { BrowserRouter, Routes, Route } from "react-router-dom";
import { CkbProvider } from "./components/CkbProvider";
import { Header } from "./components/Header";
import { HomePage } from "./pages/HomePage";
import { CreatePage } from "./pages/CreatePage";
import { ClaimPage } from "./pages/ClaimPage";
import { MyPacketsPage } from "./pages/MyPacketsPage";

function App() {
  return (
    <CkbProvider>
      <BrowserRouter>
        <div className="app">
          <Header />
          <main className="main">
            <Routes>
              <Route path="/" element={<HomePage />} />
              <Route path="/create" element={<CreatePage />} />
              <Route path="/claim" element={<ClaimPage />} />
              <Route path="/my-packets" element={<MyPacketsPage />} />
            </Routes>
          </main>
          <footer className="footer">
            <p>
              🧧 CKB Red Packet DApp — Built on{" "}
              <a
                href="https://nervos.org"
                target="_blank"
                rel="noopener noreferrer"
              >
                Nervos CKB
              </a>{" "}
              with{" "}
              <a
                href="https://github.com/ckb-devrel/ccc"
                target="_blank"
                rel="noopener noreferrer"
              >
                CCC SDK
              </a>
            </p>
          </footer>
        </div>
      </BrowserRouter>
    </CkbProvider>
  );
}

export default App;

