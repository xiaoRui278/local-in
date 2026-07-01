export interface Peer {
  peer_id: string;
  name: string;
  avatar: string;
  online: boolean;
}

export type FileTransferStatus = "pending" | "hashing" | "transferring" | "completed" | "failed" | "cancelled" | "rejected";

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
  file_status?: FileTransferStatus;
  file_progress?: number;
  received_size?: number;
  transfer_speed?: number;
  error_message?: string;
  file_path?: string;
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

export interface ChatHistoryRecord {
  peer_id: string;
  peer_name: string;
  last_message: string;
  last_message_time: number;
  record_type: "private" | "group";
  group_id?: string | null;
  member_count?: number | null;
}

export interface MessagePayload {
  record: MessageRecord;
  is_new: boolean;
}

export interface FilePayload {
  file_id: string;
  from: string;
  from_name: string;
  filename: string;
  file_path: string;
  timestamp: number;
}

export type TransferPhase = "hashing" | "transferring";

export type FileTransferEvent =
  | {
      kind: "progress";
      file_id: string;
      status: FileTransferStatus;
      phase: TransferPhase;
      received_size: number;
      total_size: number;
      speed: number;
    }
  | {
      kind: "completed";
      file_id: string;
      file_path: string;
    }
  | {
      kind: "failed";
      file_id: string;
      error_message: string;
    }
  | {
      kind: "cancelled";
      file_id: string;
    };

export type Theme = "dark" | "light";
export type ChatMode = "global" | "group";
export type FontFamilyOption = "jetbrains" | "system";
export type FontSizeOption = "12" | "14" | "16" | "18";

export interface GroupMember {
  group_id: string;
  peer_id: string;
  peer_name: string | null;
  joined_at: number;
}

export interface GroupSyncMember {
  peer_id: string;
  peer_name: string;
  joined_at: number;
}

export type GroupEventPayload =
  | {
      kind: "chat";
      group_id: string;
      passcode: string;
      group_name: string;
      creator_peer: string;
      from_peer: string;
      from_name: string;
      content: string;
      timestamp: number;
    }
  | {
      kind: "join";
      group_id: string;
      passcode: string;
      group_name: string;
      creator_peer: string;
      peer_id: string;
      peer_name: string;
      joined_at: number;
    }
  | {
      kind: "leave";
      group_id: string;
      peer_id: string;
    }
  | {
      kind: "dissolve";
      group_id: string;
    }
  | {
      kind: "sync";
      group_id: string;
      passcode: string;
      group_name: string;
      creator_peer: string;
      members: GroupSyncMember[];
    };
