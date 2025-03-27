import React from "react";
import { QrCard } from "./QrCard";

export const QrCodePanel: React.FC = () => {
  const [link, setLink] = React.useState<string | null>(null);

  React.useEffect(() => {
    const fetchCode = async () => {
      try {
        const res = await fetch("http://bruno-linux:8080/generate_code");
        const json = await res.json();
        const fullLink = `http://${json.ip}:8080?code=${json.code}`;
        setLink(fullLink);
      } catch (err) {
        console.error("Erro ao gerar QR Code:", err);
      }
    };

    fetchCode();
  }, []);

  if (!link) return <p>Gerando QR Code...</p>;

  return <QrCard value={link} />;
};
