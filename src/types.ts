export interface Note {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  content_type: "text" | "image" | "file";
  content_ref: string | null;
  file_name: string | null;
  created_at: string;
  z_index: number;
}

export interface ClipboardEvent {
  id: string;
  content_type: string;
  content: string | null;
  file_name: string | null;
  created_at: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  z_index: number;
  content_ref: string | null;
}

export const NOTE_COLORS = [
  "#FEF08A", "#FECACA", "#BFDBFE", "#BBF7D0",
  "#DDD6FE", "#FED7AA", "#FBCFE8", "#A5F3FC",
];

export const NOTE_SHADOWS = {
  "#FEF08A": "rgba(180,160,40,0.3)",
  "#FECACA": "rgba(180,60,60,0.3)",
  "#BFDBFE": "rgba(60,100,180,0.3)",
  "#BBF7D0": "rgba(60,140,80,0.3)",
  "#DDD6FE": "rgba(100,80,180,0.3)",
  "#FED7AA": "rgba(180,120,60,0.3)",
  "#FBCFE8": "rgba(180,60,130,0.3)",
  "#A5F3FC": "rgba(60,140,160,0.3)",
};

export interface Connection {
  id: string;
  from_note_id: string;
  to_note_id: string;
  color: string;
}

export const CONNECTION_COLORS = [
  "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4",
  "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
  "#BB8FCE", "#85C1E9", "#F8C471", "#82E0AA",
];
