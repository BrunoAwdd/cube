import React from "react";
import { QRCodeSVG } from "qrcode.react";
import { Stack, Text, PrimaryButton, Icon } from "@fluentui/react";

interface QrCardProps {
  value: string;
}

export const QrCard: React.FC<QrCardProps> = ({ value }) => {
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      alert("Link copiado para a área de transferência!");
    } catch (err) {
      alert("Falha ao copiar link");
    }
  };

  return (
    <Stack
      styles={{
        root: {
          backgroundColor: "#f9fafe",
          padding: 24,
          borderRadius: 8,
          boxShadow: "0 2px 6px rgba(0,0,0,0.1)",
          alignItems: "center",
        },
      }}
      tokens={{ childrenGap: 16 }}
    >
      <Text variant="xLarge">Escaneie para conectar</Text>
      <QRCodeSVG value={value} size={240} />
      <Text styles={{ root: { color: "#555" } }}>{value}</Text>
      <PrimaryButton iconProps={{ iconName: "Copy" }} onClick={handleCopy}>
        Copiar link
      </PrimaryButton>
    </Stack>
  );
};
