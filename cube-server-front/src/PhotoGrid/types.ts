export type PhotoStatus = "success" | "error" | "uploading";

export type Photo = {
  id: string;
  url: string;
  name: string;
  size: string;
  status: PhotoStatus;
};
