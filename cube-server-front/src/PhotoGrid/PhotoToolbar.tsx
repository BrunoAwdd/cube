import React from "react";
import { CommandBar, ICommandBarItemProps } from "@fluentui/react";
import { Photo } from "./types";

interface PhotoToolbarProps {
  photos: Photo[];
  selectedIds: Set<string>;
  onClear: () => void;
  onSelectAll: () => void;
  onSelectFolder: (folder: string) => Promise<string>;
  onCopy: () => void;
}

export const PhotoToolbar: React.FC<PhotoToolbarProps> = ({
  photos,
  selectedIds,
  onClear,
  onSelectAll,
  onSelectFolder,
  onCopy,
}) => {
  const itemStyles = {
    root: {
      backgroundColor: "#f9fafe",
      borderRadius: 4,
      padding: "4px 8px",
    },
    rootHovered: {
      backgroundColor: "#e4f0fb",
    },
    icon: {
      color: "#0078d4",
    },
  };

  const items: ICommandBarItemProps[] = [
    {
      key: "copy",
      text: "Copiar",
      iconProps: { iconName: "Copy" },
      onClick: onCopy,
      disabled: selectedIds.size === 0,
      buttonStyles: itemStyles,
    },
    {
      key: "selectAll",
      text: "Selecionar tudo",
      iconProps: { iconName: "SelectAll" },
      onClick: onSelectAll,
      buttonStyles: itemStyles,
    },
    {
      key: "clear",
      text: "Limpar seleção",
      iconProps: { iconName: "Cancel" },
      onClick: onClear,
      buttonStyles: itemStyles,
    },
    {
      key: "folder",
      text: "Escolher Pasta",
      iconProps: { iconName: "FolderOpen" },
      onClick: () => onSelectFolder("/home/bruno/Imagens/bruno"),
      buttonStyles: itemStyles,
    },
  ];

  return (
    <CommandBar
      items={items}
      styles={{
        root: {
          backgroundColor: "#f9fafe",
          boxShadow: "0 1px 4px rgba(0,0,0,0.08)",
        },
      }}
    />
  );
};
