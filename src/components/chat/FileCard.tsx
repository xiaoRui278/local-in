import type { MessageRecord } from "../../types";
import { FileIcon } from "../Icons";

interface FileCardProps {
  message: MessageRecord;
  isMine: boolean;
  onAccept: (fileId: string, fromPeer: string, messageId: string) => void;
}

export function FileCard({ message, isMine, onAccept }: FileCardProps) {
  const fileSizeMB = ((message.file_size || 0) / 1024 / 1024).toFixed(2);

  return (
    <div
      className={`message ${isMine ? "sent" : "received"} message-enter`}
      role="article"
      aria-label={`文件: ${message.file_name}`}
    >
      <div className="file-card">
        <div className="file-icon" aria-hidden="true">
          <FileIcon width={24} height={24} color="var(--color-accent-blue)" />
        </div>
        <div className="file-info">
          <div className="file-name">{message.file_name}</div>
          <div className="file-size">{fileSizeMB} MB</div>
        </div>
        {message.file_status === "pending" && !isMine && (
          <button
            className="file-accept-btn"
            onClick={() => onAccept(message.file_id!, message.from_peer, message.id)}
            aria-label={`接收文件 ${message.file_name}`}
          >
            接收
          </button>
        )}
        {message.file_status === "transferring" && (
          <div className="file-status" role="status">传输中...</div>
        )}
        {message.file_status === "completed" && (
          <div className="file-status" role="status">已完成</div>
        )}
      </div>
    </div>
  );
}
