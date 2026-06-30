import type { MessageRecord } from "../../types";
import { FileIcon } from "../Icons";

interface FileCardProps {
  message: MessageRecord;
  isMine: boolean;
  onAccept: (fileId: string, fromPeer: string, messageId: string) => void;
  onCancel: (fileId: string) => void;
  onRetry: (fileId: string) => void;
}

function formatBytes(bytes = 0) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

function statusLabel(status: MessageRecord["file_status"]) {
  switch (status) {
    case "hashing":
      return "校验中";
    case "transferring":
      return "传输中";
    case "completed":
      return "已完成";
    case "failed":
      return "传输失败";
    case "cancelled":
      return "已取消";
    default:
      return "待接收";
  }
}

export function FileCard({ message, isMine, onAccept, onCancel, onRetry }: FileCardProps) {
  const progress = Math.max(0, Math.min(1, message.file_progress || 0));
  const isActive = message.file_status === "transferring" || message.file_status === "hashing";
  const canRetry = !isMine && (message.file_status === "failed" || message.file_status === "cancelled");

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
          <div className="file-size">
            {formatBytes(message.received_size)} / {formatBytes(message.file_size)}
            {message.transfer_speed ? ` · ${formatBytes(message.transfer_speed)}/s` : ""}
          </div>
          {isActive && (
            <div className="file-progress" role="progressbar" aria-valuenow={Math.round(progress * 100)} aria-valuemin={0} aria-valuemax={100}>
              <div className="file-progress-fill" style={{ width: `${progress * 100}%` }} />
            </div>
          )}
          {message.error_message && <div className="file-error">{message.error_message}</div>}
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
        {isActive && message.file_id && (
          <button
            className="file-secondary-btn"
            onClick={() => onCancel(message.file_id!)}
            aria-label={`取消文件 ${message.file_name}`}
          >
            取消
          </button>
        )}
        {canRetry && message.file_id && (
          <button
            className="file-secondary-btn"
            onClick={() => onRetry(message.file_id!)}
            aria-label={`重试文件 ${message.file_name}`}
          >
            重试
          </button>
        )}
        {!isActive && message.file_status !== "pending" && (
          <div className="file-status" role="status">{statusLabel(message.file_status)}</div>
        )}
      </div>
    </div>
  );
}
