import { useState } from "react";

interface JoinGroupModalProps {
  show: boolean;
  onClose: () => void;
  onJoin: (passcode: string) => Promise<boolean>;
}

export function JoinGroupModal({ show, onClose, onJoin }: JoinGroupModalProps) {
  const [passcode, setPasscode] = useState("");
  const [error, setError] = useState("");

  if (!show) return null;

  const handleJoin = async () => {
    if (passcode.length !== 4) return;
    setError("");
    const success = await onJoin(passcode);
    if (success) {
      handleClose();
    } else {
      setError("口令无效或群聊不存在");
    }
  };

  const handleClose = () => {
    setPasscode("");
    setError("");
    onClose();
  };

  return (
    <div className="modal-overlay" onClick={handleClose} role="dialog" aria-modal="true" aria-label="加入群聊">
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h3>加入群聊</h3>
        <div className="modal-content">
          <label htmlFor="join-passcode">输入 4 位口令</label>
          <input
            id="join-passcode"
            type="text"
            value={passcode}
            onChange={(e) => {
              const val = e.target.value.replace(/\D/g, "").slice(0, 4);
              setPasscode(val);
              setError("");
            }}
            placeholder="例如：5823"
            maxLength={4}
            onKeyDown={(e) => e.key === "Enter" && handleJoin()}
            autoFocus
            aria-invalid={!!error}
            aria-describedby={error ? "passcode-error" : undefined}
          />
          {error && (
            <p id="passcode-error" className="error-text" role="alert">{error}</p>
          )}
        </div>
        <div className="modal-actions">
          <button className="btn-secondary" onClick={handleClose}>取消</button>
          <button className="btn-primary" onClick={handleJoin} disabled={passcode.length !== 4}>加入</button>
        </div>
      </div>
    </div>
  );
}
