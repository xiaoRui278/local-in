import type { Peer, ChatHistoryItem, ChatMode, Theme } from "../types";
import { SettingsIcon, SunIcon, MoonIcon, BellIcon } from "./Icons";

interface SidebarProps {
  theme: Theme;
  name: string;
  peers: Peer[];
  chatHistory: ChatHistoryItem[];
  chatMode: ChatMode;
  selectedPeer: string | null;
  selectedGroup: string | null;
  onToggleTheme: () => void;
  onOpenSettings: () => void;
  onSelectGlobal: () => void;
  onSelectPrivate: (peerId: string) => void;
  onSelectGroup: (groupId: string) => void;
  onCreateGroup: () => void;
  onJoinGroup: () => void;
}

function getAvatarColor(name: string) {
  const colors = [
    "var(--avatar-gradient-1)",
    "var(--avatar-gradient-2)",
    "var(--avatar-gradient-3)",
    "var(--avatar-gradient-4)",
    "var(--avatar-gradient-5)",
  ];
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  return colors[Math.abs(hash) % colors.length];
}

function formatLastMessage(message: string) {
  if (!message) return "新消息";
  if (!message.startsWith("[FILE]")) return message;

  const parts = message.slice(6).split("|");
  const filename = parts[1];
  return filename ? `文件：${filename}` : "文件消息";
}

export function Sidebar({
  theme, name, peers, chatHistory, chatMode, selectedPeer, selectedGroup,
  onToggleTheme, onOpenSettings, onSelectGlobal, onSelectPrivate, onSelectGroup,
  onCreateGroup, onJoinGroup,
}: SidebarProps) {
  return (
    <nav className="sidebar" aria-label="聊天列表">
      <div className="sidebar-header">
        <div className="logo-small" aria-hidden="true">LI</div>
        <span className="current-user-name">{name}</span>
        <div className="header-actions">
          <button
            className="icon-btn"
            onClick={onOpenSettings}
            aria-label="设置"
          >
            <SettingsIcon />
          </button>
          <button
            className="icon-btn"
            onClick={onToggleTheme}
            aria-label={theme === "dark" ? "切换到浅色模式" : "切换到深色模式"}
          >
            {theme === "dark" ? <SunIcon /> : <MoonIcon />}
          </button>
        </div>
      </div>

      <div className="sidebar-section sidebar-fixed">
        <div
          className={`peer-item ${chatMode === "global" && !selectedPeer ? "selected" : ""}`}
          onClick={onSelectGlobal}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => e.key === "Enter" && onSelectGlobal()}
          aria-current={chatMode === "global" && !selectedPeer ? "true" : undefined}
        >
          <div className="avatar avatar-bell" aria-hidden="true">
            <BellIcon width={16} height={16} color="white" />
          </div>
          <div className="peer-info">
            <span className="peer-name">公共频道</span>
            <span className="peer-status">{peers.length} 在线</span>
          </div>
        </div>
      </div>

      <div className="section-label" role="heading" aria-level={2}>我的聊天</div>
      <div className="chat-list" role="list">
        {chatHistory.length === 0 ? (
          <div className="empty-state" style={{ padding: "var(--space-3)" }}>
            <p style={{ fontSize: "var(--font-size-base)", opacity: 0.5 }}>暂无聊天记录</p>
          </div>
        ) : (
          chatHistory.map((item) => {
            const key = item.type === "group" ? item.group_id : item.peer_id;
            const isSelected =
              (item.type === "group" && selectedGroup === item.group_id) ||
              (item.type === "private" && selectedPeer === item.peer_id && chatMode === "global");
            return (
              <div
                key={key}
                className={`peer-item ${isSelected ? "selected" : ""}`}
                onClick={() => {
                  if (item.type === "group") {
                    onSelectGroup(item.group_id!);
                  } else {
                    onSelectPrivate(item.peer_id);
                  }
                }}
                role="listitem"
                tabIndex={0}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    if (item.type === "group") onSelectGroup(item.group_id!);
                    else onSelectPrivate(item.peer_id);
                  }
                }}
                aria-current={isSelected ? "true" : undefined}
              >
                <div className="avatar" style={{ background: getAvatarColor(item.peer_name) }} aria-hidden="true">
                  {item.peer_name[0]}
                </div>
                <div className="peer-info">
                  <span className="peer-name">{item.peer_name}</span>
                  <span className="peer-status">
                    {item.type === "group" ? `${item.member_count} 人` : formatLastMessage(item.last_message)}
                  </span>
                </div>
              </div>
            );
          })
        )}
      </div>

      <div className="group-actions">
        <button className="btn-secondary" onClick={onCreateGroup} aria-label="创建群聊">
          创建群聊
        </button>
        <button className="btn-secondary" onClick={onJoinGroup} aria-label="加入群聊">
          加入群聊
        </button>
      </div>
    </nav>
  );
}
