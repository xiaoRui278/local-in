import { SendIcon, PaperclipIcon } from "../Icons";

const MAX_MESSAGE_LENGTH = 2000;

interface InputAreaProps {
  value: string;
  onChange: (value: string) => void;
  onSend: () => void;
  onFileSelect?: () => void;
  showFileButton?: boolean;
  placeholder?: string;
  disabled?: boolean;
}

export function InputArea({
  value, onChange, onSend, onFileSelect, showFileButton = false, placeholder = "输入消息...", disabled = false
}: InputAreaProps) {
  const showCounter = value.length >= MAX_MESSAGE_LENGTH - 200;
  return (
    <div className="input-area">
      {showFileButton && onFileSelect && (
        <button className="icon-btn" onClick={onFileSelect} aria-label="发送文件" disabled={disabled}>
          <PaperclipIcon width={20} height={20} />
        </button>
      )}
      <input
        type="text"
        placeholder={disabled ? "对方已离线，无法发送" : placeholder}
        value={value}
        maxLength={MAX_MESSAGE_LENGTH}
        onChange={(e) => onChange(e.target.value.slice(0, MAX_MESSAGE_LENGTH))}
        onKeyDown={(e) => e.key === "Enter" && !disabled && onSend()}
        aria-label="消息输入"
        disabled={disabled}
      />
      {showCounter && (
        <span className="input-counter" aria-live="polite">{value.length}/{MAX_MESSAGE_LENGTH}</span>
      )}
      <button className="send-btn" onClick={onSend} aria-label="发送消息" disabled={disabled}>
        <SendIcon />
      </button>
    </div>
  );
}
