import type { MessageRecord } from "../../types";

interface MessageBubbleProps {
  message: MessageRecord;
  isMine: boolean;
  showSender: boolean;
  formatTime: (timestamp: number) => string;
}

export function MessageBubble({ message, isMine, showSender, formatTime }: MessageBubbleProps) {
  return (
    <div
      className={`message ${isMine ? "sent" : "received"} message-enter`}
      role="article"
      aria-label={`${message.from_name} 说: ${message.content}`}
    >
      {showSender && (
        <div className="message-sender">{message.from_name}</div>
      )}
      <div className="message-content">{message.content}</div>
      <div className="message-time">
        <time dateTime={new Date(message.timestamp * 1000).toISOString()}>
          {formatTime(message.timestamp)}
        </time>
      </div>
    </div>
  );
}
