import React from "react";
import { Stack, Text } from "@fluentui/react";
import { Photo } from "./types";
import { useSelection } from "./useSelection";
import { PhotoItem } from "./PhotoItem";
import { PhotoToolbar } from "./PhotoToolbar";

// Mock de fotos com os campos atualizados
const mockPhotos: Photo[] = Array.from({ length: 50 }, (_, i) => ({
  id: `photo-${i}`,
  url: `https://picsum.photos/300/300?random=${i}`,
  name: `imagem_${i + 1}.jpg`,
  size: `${(Math.random() * 4 + 1).toFixed(2)} MB`,
  status:
    Math.random() > 0.7
      ? "error"
      : Math.random() > 0.4
      ? "success"
      : "uploading",
}));

export const PhotoGrid: React.FC = () => {
  const {
    selectedIds,
    toggleSelection,
    clearSelection,
    isSelected,
    selectAll,
  } = useSelection(mockPhotos);

  const sendConfigToServer = async (folderPath: string) => {
    console.log("Enviando para", folderPath);
    try {
      await fetch("http://bruno-linux:8080/set-config", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ folder: folderPath }),
      });
      console.log("✅ Configuração enviada com sucesso");
    } catch (error) {
      console.error("❌ Falha ao enviar config:", error);
    }
  };

  return (
    <Stack tokens={{ childrenGap: 10 }} styles={{ root: { padding: 16 } }}>
      <PhotoToolbar
        photos={mockPhotos}
        selectedIds={selectedIds}
        onClear={clearSelection}
        onSelectAll={selectAll}
        onSelectFolder={sendConfigToServer}
      />
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: 12,
          justifyContent: "flex-start",
        }}
      >
        {mockPhotos.map((photo) => (
          <PhotoItem
            key={photo.id}
            photo={photo}
            selected={isSelected(photo.id)}
            onClick={() => toggleSelection(photo.id)}
          />
        ))}
      </div>
      {mockPhotos.length === 0 && <Text>Sem fotos para exibir.</Text>}
    </Stack>
  );
};
