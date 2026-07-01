import { useState } from "react";
import type { Theme } from "../types";
import { SunIcon, MoonIcon } from "./Icons";

interface LoginScreenProps {
  theme: Theme;
  onToggleTheme: () => void;
  onStart: (name: string) => void;
}

export function LoginScreen({ theme, onToggleTheme, onStart }: LoginScreenProps) {
  const [name, setName] = useState("");

  const handleSubmit = () => {
    if (name.trim()) onStart(name.trim());
  };

  return (
    <div className="login-container" role="main">
      <div className="login-card">
        <div className="logo-container">
          <div className="logo" aria-hidden="true">LI</div>
        </div>
        <h1>Local-In</h1>
        <p className="subtitle">局域网P2P聊天</p>
        <div className="input-group">
          <label htmlFor="nickname-input" className="sr-only">昵称</label>
          <input
            id="nickname-input"
            type="text"
            placeholder="输入你的昵称"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
            autoComplete="off"
            autoFocus
          />
        </div>
        <button className="btn-primary" onClick={handleSubmit} disabled={!name.trim()}>
          加入
        </button>
        <button
          className="theme-toggle"
          onClick={onToggleTheme}
          aria-label={theme === "dark" ? "切换到浅色模式" : "切换到深色模式"}
        >
          {theme === "dark" ? <SunIcon width={18} height={18} /> : <MoonIcon width={18} height={18} />}
        </button>
      </div>
    </div>
  );
}
