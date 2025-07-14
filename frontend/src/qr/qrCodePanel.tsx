import Database from "@tauri-apps/plugin-sql";
import { QrCard } from "./QrCard";
import React from "react";
import { invoke } from "@tauri-apps/api/core";

export const QrCodePanel: React.FC = () => {
  const [link, setLink] = React.useState<string | null>(null);

   
  React.useEffect(() => {
    async function get_qr_code() {
      const db = await Database.load("sqlite:uploads.db");
      try {
        const qrVars = await invoke<{ code: string; ip: string; expires_in: number }>(
          "get_qr_code"
        );

        const { code, ip, expires_in } = qrVars;

        await db.execute(
          "INSERT INTO auth_codes (code, ip, created_at) VALUES (?, ?, ?)",
          [code, ip, expires_in],
        );

        setLink(`http://${ip}:8080?code=${code}`);
      } catch (err) {
        console.error("❌ Erro ao invocar comando get_qr_code:", err);
      }
    }
    if (!link) {
      get_qr_code();
    }
  }, []);

  if (!link) return <p>Gerando QR Code...</p>;

  return <QrCard value={link} />;
};
