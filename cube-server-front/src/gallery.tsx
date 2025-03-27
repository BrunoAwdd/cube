import * as React from "react";
import {
  Stack,
  Image,
  ImageFit,
  Icon,
  CommandBar,
  ICommandBarItemProps,
  mergeStyleSets,
  Text,
} from "@fluentui/react";

type Photo = {
  id: string;
  url: string;
  name: string;
  size: string;
  status: "success" | "error" | "uploading";
};

const photos: Photo[] = Array.from({ length: 40 }).map((_, i) => ({
  id: `photo-${i}`,
  url: `https://picsum.photos/seed/${i}/300/300`,
  name: `IMG_${i + 1000}.JPG`,
  size: `${(Math.random() * 3 + 0.5).toFixed(1)} MB`,
  status: i % 5 === 0 ? "error" : i % 3 === 0 ? "uploading" : "success",
}));

const classNames = mergeStyleSets({
  container: {
    padding: 12,
  },
  itemWrap: {
    width: 120,
    textAlign: "center",
    cursor: "pointer",
    selectors: {
      "&:hover .overlay": {
        opacity: 1,
      },
    },
  },
  photoWrap: {
    position: "relative",
    width: 120,
    height: 120,
    borderRadius: 4,
    overflow: "hidden",
    border: "2px solid transparent",
  },
  selected: {
    borderColor: "#0078d4",
  },
  statusIcon: {
    position: "absolute",
    top: 6,
    right: 6,
    zIndex: 1,
  },
  checkIcon: {
    position: "absolute",
    top: 6,
    left: 6,
    zIndex: 1,
    background: "#fff",
    borderRadius: "50%",
    padding: 2,
  },
  caption: {
    marginTop: 6,
    fontSize: 12,
    color: "#555",
    whiteSpace: "nowrap",
    overflow: "hidden",
    textOverflow: "ellipsis",
  },
});

export const Gallery: React.FC = () => {
  const [filter, setFilter] = React.useState<
    "all" | "success" | "error" | "uploading"
  >("all");
  const [selectedIds, setSelectedIds] = React.useState<Set<string>>(new Set());

  const toggleSelection = (id: string) => {
    setSelectedIds((prev) => {
      const newSet = new Set(prev);
      newSet.has(id) ? newSet.delete(id) : newSet.add(id);
      return newSet;
    });
  };

  const filteredPhotos = photos.filter((photo) =>
    filter === "all" ? true : photo.status === filter
  );

  const getIcon = (status: string) => {
    switch (status) {
      case "success":
        return <Icon iconName="CheckMark" style={{ color: "green" }} />;
      case "error":
        return <Icon iconName="ErrorBadge" style={{ color: "red" }} />;
      case "uploading":
        return <Icon iconName="Sync" className="spin" />;
      default:
        return null;
    }
  };

  const handleCopy = async () => {
    const selected = photos.filter((p) => selectedIds.has(p.id));
    for (const p of selected) {
      const blob = await fetch(p.url).then((res) => res.blob());
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = p.name;
      a.click();
    }
  };

  const commandBarItems: ICommandBarItemProps[] = [
    { key: "all", text: "Todos", onClick: () => setFilter("all") },
    { key: "success", text: "✅ OK", onClick: () => setFilter("success") },
    { key: "error", text: "❌ Erro", onClick: () => setFilter("error") },
    {
      key: "uploading",
      text: "⏳ Enviando",
      onClick: () => setFilter("uploading"),
    },
  ];

  return (
    <Stack className={classNames.container} tokens={{ childrenGap: 12 }}>
      <CommandBar
        items={commandBarItems}
        farItems={[
          {
            key: "copy",
            text: `Copiar (${selectedIds.size})`,
            iconOnly: false,
            iconProps: { iconName: "Copy" },
            onClick: handleCopy,
            disabled: selectedIds.size === 0,
          },
        ]}
      />
      <Stack horizontal wrap tokens={{ childrenGap: 12 }}>
        {filteredPhotos.map((photo) => {
          const isSelected = selectedIds.has(photo.id);

          return (
            <Stack
              key={photo.id}
              className={classNames.itemWrap}
              onClick={() => toggleSelection(photo.id)}
            >
              <div
                className={`${classNames.photoWrap} ${
                  isSelected ? classNames.selected : ""
                }`}
              >
                <Image
                  src={photo.url}
                  width={120}
                  height={120}
                  imageFit={ImageFit.cover}
                />
                <div className={classNames.statusIcon}>
                  {getIcon(photo.status)}
                </div>
                {isSelected && (
                  <div className={classNames.checkIcon}>
                    <Icon iconName="CheckboxComposite" />
                  </div>
                )}
              </div>
              <Text className={classNames.caption} title={photo.name}>
                {photo.name}
              </Text>
              <Text className={classNames.caption}>{photo.size}</Text>
            </Stack>
          );
        })}
      </Stack>
    </Stack>
  );
};
