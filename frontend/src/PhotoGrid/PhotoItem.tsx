import React from "react";
import {
  Image,
  ImageFit,
  Icon,
  Stack,
  Text,
  mergeStyleSets,
} from "@fluentui/react";
import { Photo } from "./types";

const classNames = mergeStyleSets({
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

type Props = {
  photo: Photo;
  selected: boolean;
  onClick: () => void;
};

export const PhotoItem: React.FC<Props> = ({ photo, selected, onClick }) => {
  const getIcon = () => {
    switch (photo.status) {
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

  return (
    <Stack className={classNames.itemWrap} onClick={onClick}>
      <div
        className={`${classNames.photoWrap} ${
          selected ? classNames.selected : ""
        }`}
      >
        <Image
          src={`http://bruno-linux:8080${photo.url}`}
          width={120}
          height={120}
          imageFit={ImageFit.cover}
        />
        <div className={classNames.statusIcon}>{getIcon()}</div>
        {selected && (
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
};
