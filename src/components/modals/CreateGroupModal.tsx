import { useState } from "react";
import { CopyIcon } from "../Icons";

interface CreateGroupModalProps {
  show: boolean;
  onClose: () => void;
  onCreate: (name: string) => Promise<{ passcode: string } | null>;
}

export function CreateGroupModal({ show, onClose, onCreate }: CreateGroupModalProps) {
  const [name, setName] = useState("");
  const [passcode, setPasscode] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  if (!show) return null;

  const handleCreate = async () => {
    if (!name.trim()) return;
    const result = await onCreate(name.trim());
    if (result) {
      setPasscode(result.passcode);
    }
  };

  const handleCopy = () => {
    if (passcode) {
      navigator.clipboard.writeText(passcode);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleClose = () => {
    setName("");
    setPasscode(null);
    setCopied(false);
    onClose();
  };

  return (
    <div className="modal-overlay" onClick={handleClose} role="dialog" aria-modal="true" aria-label="创建群聊">
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h3>{passcode ? "群聊已创建" : "创建群聊"}</h3>
        {!passcode ? (
          <>
            <div className="modal-content">
              <label htmlFor="group-name">群名称</label>
              <input
                id="group-name"
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="输入群名称"
                onKeyDown={(e) => e.key === "Enter" && handleCreate()}
                autoFocus
              />
            </div>
            <div className="modal-actions">
              <button className="btn-secondary" onClick={handleClose}>取消</button>
              <button className="btn-primary" onClick={handleCreate} disabled={!name.trim()}>创建</button>
            </div>
          </>
        ) : (
          <>
            <div className="modal-content" style={{ textAlign: "center" }}>
              <p style={{ marginBottom: "var(--space-3)" }}>分享此口令邀请好友加入：</p>
              <div className="passcode-display">
                {passcode}
              </div>
              <button
                className="btn-secondary"
                style={{ marginTop: "var(--space-3)", fontSize: "var(--font-size-base)" }}
                onClick={handleCopy}
              >
                <CopyIcon width={14} height={14} />
                <span style={{ marginLeft: "var(--space-1)" }}>{copied ? "已复制" : "复制口令"}</span>
              </button>
            </div>
            <div className="modal-actions">
              <button className="btn-primary" onClick={handleClose}>完成</button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
