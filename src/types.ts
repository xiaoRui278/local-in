export interface Peer {
  peer_id: string;
  name: string;
  avatar: string;
  online: boolean;
}

export interface MessageRecord {
  id: string;
  from_peer: string;
  from_name: string;
  to_peer: string;
  content: string;
  timestamp: number;
  is_read: boolean;
  file_id?: string;
  file_name?: string;
  file_size?: number;
  file_status?: "pending" | "transferring" | "completed";
}

export interface GroupInfo {
  id: string;
  name: string;
  passcode: string;
  creator_peer: string;
  member_count: number;
}

export interface GroupMessageRecord {
  id: string;
  group_id: string;
  from_peer: string;
  from_name: string;
  content: string;
  timestamp: number;
}

export interface ChatHistoryItem {
  peer_id: string;
  peer_name: string;
  last_message: string;
  last_message_time: number;
  type: "private" | "group";
  group_id?: string;
  member_count?: number;
}

export interface MessagePayload {
  record: MessageRecord;
  is_new: boolean;
}

export interface FilePayload {
  from: string;
  from_name: string;
  filename: string;
  file_path: string;
  timestamp: number;
}

export type Theme = "dark" | "light";
export type ChatMode = "global" | "group";
export type FontFamilyOption = "jetbrains" | "system";
export type FontSizeOption = "12" | "14" | "16" | "18";
