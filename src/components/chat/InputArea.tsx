import { SendIcon, PaperclipIcon } from "../Icons";

interface InputAreaProps {
  value: string;
  onChange: (value: string) => void;
  onSend: () => void;
  onFileSelect?: () => void;
  showFileButton?: boolean;
  placeholder?: string;
}

export function InputArea({
  value, onChange, onSend, onFileSelect, showFileButton = false, placeholder = "输入消息..."
}: InputAreaProps) {
  return (
    <div className="input-area">
      {showFileButton && onFileSelect && (
        <button className="icon-btn" onClick={onFileSelect} aria-label="发送文件">
          <PaperclipIcon width={20} height={20} />
        </button>
      )}
      <input
        type="text"
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && onSend()}
        aria-label="消息输入"
      />
      <button className="send-btn" onClick={onSend} aria-label="发送消息">
        <SendIcon />
      </button>
    </div>
  );
}
