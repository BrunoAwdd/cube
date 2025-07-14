import React, { useEffect, useState } from "react";
import { Stack, Text } from "@fluentui/react";

import { Photo } from "./types";
import { PhotoItem } from "./PhotoItem";
import { PhotoToolbar } from "./PhotoToolbar";
import { invoke } from "@tauri-apps/api/core";
import { open } from '@tauri-apps/plugin-dialog';
import { useSelection } from "./useSelection";

export const PhotoGrid: React.FC = ({
  send,
}: {
  send: (data: any) => void;
}) => {
  const [photos, setPhotos] = useState<Photo[]>([]);
  const [link, setLink] = useState<string | null>(null);
  const {
    selectedIds,
    toggleSelection,
    clearSelection,
    isSelected,
    selectAll,
  } = useSelection(photos);


  async function sendConfig (): Promise<string> {
    try {
      const path = await open({
        multiple: false,
        directory: true,
      });

      return await invoke<string>("set_config", { payload: {upload_dir: path} });
    } catch (error) {
      console.error("❌ Falha ao enviar config:", error); 
      return "Erro ao enviar configuração";
    }
  }

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
        onSelectFolder={sendConfig}
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
