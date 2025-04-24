import React, { useEffect, useState } from "react";
import { Stack, Text } from "@fluentui/react";
import { Photo } from "./types";
import { useSelection } from "./useSelection";
import { PhotoItem } from "./PhotoItem";
import { PhotoToolbar } from "./PhotoToolbar";

export const PhotoGrid: React.FC = ({
  send,
}: {
  send: (data: any) => void;
}) => {
  const [photos, setPhotos] = useState<Photo[]>([]);
  const {
    selectedIds,
    toggleSelection,
    clearSelection,
    isSelected,
    selectAll,
  } = useSelection(photos);

  // 🔄 Carrega fotos da API
  useEffect(() => {
    const loadPhotos = async () => {
      try {
        const res = await fetch("http://bruno-linux:8080/api/thumbs/list");
        const data = await res.json();
        setPhotos(data);
      } catch (err) {
        console.error("❌ Erro ao carregar thumbs:", err);
      }
    };

    loadPhotos();
  }, []);

  const sendConfigToServer = async (folderPath: string): Promise<string> => {
    try {
      const response = await fetch("http://bruno-linux:8080/set-config", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ folder: folderPath }),
      });
      console.log("✅ Configuração enviada com sucesso");

      return response.json();
    } catch (error) {
      console.error("❌ Falha ao enviar config:", error);
    }
  };

  const onCopy = () => {
    send({
      type: "action",
      name: "copy_files",
      payload: {
        hashes: Array.from(selectedIds),
      },
    });
  };

  return (
    <Stack tokens={{ childrenGap: 10 }} styles={{ root: { padding: 16 } }}>
      <PhotoToolbar
        photos={photos}
        selectedIds={selectedIds}
        onClear={clearSelection}
        onSelectAll={selectAll}
        onSelectFolder={sendConfigToServer}
        onCopy={onCopy}
      />
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: 12,
          justifyContent: "flex-start",
        }}
      >
        {photos.map((photo) => (
          <PhotoItem
            key={photo.id}
            photo={photo}
            selected={isSelected(photo.id)}
            onClick={() => toggleSelection(photo.id)}
          />
        ))}
      </div>
      {photos.length === 0 && <Text>Sem fotos para exibir.</Text>}
    </Stack>
  );
};
