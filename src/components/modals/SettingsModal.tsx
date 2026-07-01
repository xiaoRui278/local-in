import { useState } from "react";
import type { Theme, FontFamilyOption, FontSizeOption } from "../../types";

interface SettingsModalProps {
  show: boolean;
  theme: Theme;
  currentName: string;
  currentFont: FontFamilyOption;
  currentSize: FontSizeOption;
  onClose: () => void;
  onSave: (name: string, font: FontFamilyOption, size: FontSizeOption) => void;
  onToggleTheme: () => void;
}

export function SettingsModal({
  show, theme, currentName, currentFont, currentSize, onClose, onSave, onToggleTheme,
}: SettingsModalProps) {
  const [name, setName] = useState(currentName);
  const [font, setFont] = useState(currentFont);
  const [size, setSize] = useState(currentSize);

  if (!show) return null;

  const handleSave = () => {
    onSave(name, font, size);
  };

  return (
    <div className="modal-overlay" onClick={onClose} role="dialog" aria-modal="true" aria-label="设置">
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h3>设置</h3>
        <div className="modal-content">
          <label htmlFor="settings-name">昵称</label>
          <input
            id="settings-name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="输入新昵称"
            autoFocus
          />

          <label htmlFor="settings-font" style={{ marginTop: "var(--space-3)" }}>字体</label>
          <select
            id="settings-font"
            value={font}
            onChange={(e) => setFont(e.target.value as FontFamilyOption)}
            className="modal-select"
          >
            <option value="jetbrains">JetBrains Mono</option>
            <option value="system">系统字体</option>
          </select>

          <label htmlFor="settings-size" style={{ marginTop: "var(--space-3)" }}>字体大小</label>
          <select
            id="settings-size"
            value={size}
            onChange={(e) => setSize(e.target.value as FontSizeOption)}
            className="modal-select"
          >
            <option value="12">小 (12px)</option>
            <option value="14">中 (14px)</option>
            <option value="16">大 (16px)</option>
            <option value="18">特大 (18px)</option>
          </select>

          <button
            className="btn-secondary"
            style={{ marginTop: "var(--space-4)", width: "100%" }}
            onClick={onToggleTheme}
          >
            {theme === "dark" ? "切换到浅色模式" : "切换到深色模式"}
          </button>
        </div>
        <div className="modal-actions">
          <button className="btn-secondary" onClick={onClose}>取消</button>
          <button className="btn-primary" onClick={handleSave}>保存</button>
        </div>
      </div>
    </div>
  );
}
