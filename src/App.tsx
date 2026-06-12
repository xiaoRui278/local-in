import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Peer {
  peer_id: string;
  name: string;
  avatar: string;
  online: boolean;
}

interface MessageRecord {
  id: string;
  from_peer: string;
  to_peer: string;
  content: string;
  timestamp: number;
  is_read: boolean;
}

interface GroupInfo {
  id: string;
  name: string;
  passcode: string;
  creator_peer: string;
  member_count: number;
}

interface GroupMessageRecord {
  id: string;
  group_id: string;
  from_peer: string;
  from_name: string;
  content: string;
  timestamp: number;
}

function App() {
  const [name, setName] = useState("");
  const [started, setStarted] = useState(false);
  const [peers, setPeers] = useState<Peer[]>([]);
  const [messages, setMessages] = useState<MessageRecord[]>([]);
  const [input, setInput] = useState("");
  const [selectedPeer, setSelectedPeer] = useState<string | null>(null);
  const [myPeerId, setMyPeerId] = useState("");
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [showSettings, setShowSettings] = useState(false);
  const [editName, setEditName] = useState("");

  const [groups, setGroups] = useState<GroupInfo[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [groupMessages, setGroupMessages] = useState<GroupMessageRecord[]>([]);
  const [showCreateGroup, setShowCreateGroup] = useState(false);
  const [showJoinGroup, setShowJoinGroup] = useState(false);
  const [newGroupName, setNewGroupName] = useState("");
  const [joinPasscode, setJoinPasscode] = useState("");
  const [createdPasscode, setCreatedPasscode] = useState<string | null>(null);
  const [chatMode, setChatMode] = useState<"global" | "group">("global");

  useEffect(() => {
    loadSavedConfig();
  }, []);

  useEffect(() => {
    document.body.className = theme;
  }, [theme]);

  useEffect(() => {
    if (started) {
      const interval = setInterval(async () => {
        try {
          const peerList = await invoke<Peer[]>("get_peers");
          setPeers(peerList);
        } catch (e) {
          console.error("Failed to get peers:", e);
        }
      }, 2000);

      return () => clearInterval(interval);
    }
  }, [started]);

  useEffect(() => {
    if (selectedPeer) {
      loadMessages(selectedPeer);
    }
  }, [selectedPeer]);

  useEffect(() => {
    if (started) {
      loadGroups();
    }
  }, [started]);

  useEffect(() => {
    if (selectedGroup && chatMode === "group") {
      loadGroupMessages(selectedGroup);
    }
  }, [selectedGroup, chatMode]);

  const loadSavedConfig = async () => {
    try {
      const [savedName, _savedAvatar] = await invoke<[string | null, string | null]>("get_saved_config");
      if (savedName) setName(savedName);
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  };

  const loadMessages = async (peerId: string) => {
    try {
      const msgs = await invoke<MessageRecord[]>("get_messages", {
        peerId,
        limit: 100,
      });
      setMessages(msgs.reverse());
    } catch (e) {
      console.error("Failed to load messages:", e);
    }
  };

  const handleStart = async () => {
    if (!name.trim()) return;
    try {
      const peerId = await invoke<string>("start_node", { name: name.trim() });
      setMyPeerId(peerId);
      setStarted(true);
    } catch (e) {
      console.error("Failed to start node:", e);
    }
  };

  const handleSend = async () => {
    if (!input.trim() || !selectedPeer) return;
    try {
      await invoke("send_message", {
        from: myPeerId,
        to: selectedPeer,
        content: input.trim(),
      });
      setMessages((prev) => [
        ...prev,
        {
          id: Date.now().toString(),
          from_peer: myPeerId,
          to_peer: selectedPeer,
          content: input.trim(),
          timestamp: Math.floor(Date.now() / 1000),
          is_read: true,
        },
      ]);
      setInput("");
    } catch (e) {
      console.error("Failed to send message:", e);
    }
  };

  const handleUpdateName = async () => {
    if (!editName.trim()) return;
    try {
      await invoke("update_name", { newName: editName.trim() });
      setName(editName.trim());
      setShowSettings(false);
    } catch (e) {
      console.error("Failed to update name:", e);
    }
  };

  const handleFileSelect = async () => {
    if (!selectedPeer) return;

    try {
      const file = await open({
        multiple: false,
        filters: [{ name: "All Files", extensions: ["*"] }],
      });

      if (file) {
        const filePath = typeof file === "string" ? file : (file as { path: string }).path;
        await invoke<string>("send_file", {
          peerId: selectedPeer,
          filePath: filePath,
        });
      }
    } catch (e) {
      console.error("Failed to send file:", e);
    }
  };

  const loadGroups = async () => {
    try {
      const groupList = await invoke<GroupInfo[]>("get_groups");
      setGroups(groupList);
    } catch (e) {
      console.error("Failed to get groups:", e);
    }
  };

  const loadGroupMessages = async (groupId: string) => {
    try {
      const msgs = await invoke<GroupMessageRecord[]>("get_group_messages_cmd", {
        groupId,
        limit: 100,
      });
      setGroupMessages(msgs.reverse());
    } catch (e) {
      console.error("Failed to get group messages:", e);
    }
  };

  const handleCreateGroup = async () => {
    if (!newGroupName.trim()) return;
    try {
      const group = await invoke<GroupInfo>("create_group", { name: newGroupName.trim() });
      setCreatedPasscode(group.passcode);
      setGroups((prev) => [group, ...prev]);
      setNewGroupName("");
    } catch (e) {
      console.error("Failed to create group:", e);
    }
  };

  const handleJoinGroup = async () => {
    if (!joinPasscode.trim() || joinPasscode.length !== 4) return;
    try {
      const group = await invoke<GroupInfo>("join_group", { passcode: joinPasscode.trim() });
      setGroups((prev) => {
        const exists = prev.find((g) => g.id === group.id);
        if (exists) return prev;
        return [group, ...prev];
      });
      setJoinPasscode("");
      setShowJoinGroup(false);
      setSelectedGroup(group.id);
      setChatMode("group");
    } catch (e) {
      console.error("Failed to join group:", e);
    }
  };

  const handleSendGroupMessage = async () => {
    if (!input.trim() || !selectedGroup) return;
    try {
      await invoke("send_group_message_cmd", {
        groupId: selectedGroup,
        content: input.trim(),
      });
      setGroupMessages((prev) => [
        ...prev,
        {
          id: Date.now().toString(),
          group_id: selectedGroup,
          from_peer: myPeerId,
          from_name: name,
          content: input.trim(),
          timestamp: Math.floor(Date.now() / 1000),
        },
      ]);
      setInput("");
    } catch (e) {
      console.error("Failed to send group message:", e);
    }
  };

  const handleDissolveGroup = async () => {
    if (!selectedGroup) return;
    try {
      await invoke("dissolve_group", { groupId: selectedGroup });
      setGroups((prev) => prev.filter((g) => g.id !== selectedGroup));
      setSelectedGroup(null);
      setChatMode("global");
    } catch (e) {
      console.error("Failed to dissolve group:", e);
    }
  };

  const handleLeaveGroup = async () => {
    if (!selectedGroup) return;
    try {
      await invoke("leave_group", { groupId: selectedGroup });
      setGroups((prev) => prev.filter((g) => g.id !== selectedGroup));
      setSelectedGroup(null);
      setChatMode("global");
    } catch (e) {
      console.error("Failed to leave group:", e);
    }
  };

  // Suppress unused warnings - will be used in Task 8
  void groupMessages;
  void handleSendGroupMessage, handleDissolveGroup, handleLeaveGroup;

  const getAvatarColor = (name: string) => {
    const colors = [
      "linear-gradient(135deg, #F59E0B, #EF4444)",
      "linear-gradient(135deg, #10B981, #059669)",
      "linear-gradient(135deg, #8B5CF6, #6366F1)",
      "linear-gradient(135deg, #EC4899, #F43F5E)",
      "linear-gradient(135deg, #06B6D4, #0EA5E9)",
    ];
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
      hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    return colors[Math.abs(hash) % colors.length];
  };

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
  };

  if (!started) {
    return (
      <div className={`app ${theme}`}>
        <div className="login-container">
          <div className="login-card">
            <div className="logo-container">
              <div className="logo">LI</div>
            </div>
            <h1>Local-In</h1>
            <p className="subtitle">局域网P2P聊天</p>
            <div className="input-group">
              <input
                type="text"
                placeholder="输入你的昵称"
                value={name}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleStart()}
              />
            </div>
            <button className="btn-primary" onClick={handleStart}>
              加入
            </button>
            <button
              className="theme-toggle"
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            >
              {theme === "dark" ? "☀️" : "🌙"}
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={`app ${theme}`}>
      <div className="sidebar">
        <div className="sidebar-header">
          <div className="logo-small">LI</div>
          <span className="current-user-name">{name}</span>
          <div className="header-actions">
            <button
              className="icon-btn"
              onClick={() => {
                setEditName(name);
                setShowSettings(true);
              }}
              title="设置"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
              </svg>
            </button>
            <button
              className="icon-btn"
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
              title="切换主题"
            >
              {theme === "dark" ? (
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <circle cx="12" cy="12" r="5"></circle>
                  <line x1="12" y1="1" x2="12" y2="3"></line>
                  <line x1="12" y1="21" x2="12" y2="23"></line>
                  <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                  <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                  <line x1="1" y1="12" x2="3" y2="12"></line>
                  <line x1="21" y1="12" x2="23" y2="12"></line>
                  <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                  <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
                </svg>
              ) : (
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
                </svg>
              )}
            </button>
          </div>
        </div>

        <div className="peer-list">
          <div
            className={`peer-item ${chatMode === "global" && !selectedPeer ? "selected" : ""}`}
            onClick={() => {
              setChatMode("global");
              setSelectedGroup(null);
              setSelectedPeer(null);
            }}
          >
            <div className="avatar" style={{ background: "linear-gradient(135deg, #06B6D4, #0EA5E9)" }}>
              G
            </div>
            <div className="peer-info">
              <span className="peer-name">公共频道</span>
              <span className="peer-status">所有人</span>
            </div>
          </div>

          <div className="section-label" style={{ marginTop: "16px" }}>在线设备</div>
          {peers.length === 0 ? (
            <div className="empty-state">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.3">
                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                <circle cx="9" cy="7" r="4"></circle>
                <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
                <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
              </svg>
              <p>等待其他设备加入...</p>
            </div>
          ) : (
            peers.map((peer) => (
              <div
                key={peer.peer_id}
                className={`peer-item ${selectedPeer === peer.peer_id && chatMode === "global" ? "selected" : ""}`}
                onClick={() => {
                  setSelectedPeer(peer.peer_id);
                  setChatMode("global");
                  setSelectedGroup(null);
                }}
              >
                <div
                  className="avatar"
                  style={{ background: getAvatarColor(peer.name) }}
                >
                  {peer.name[0]}
                </div>
                <div className="peer-info">
                  <span className="peer-name">{peer.name}</span>
                  <span className="peer-status">
                    {peer.online ? "在线" : "离线"}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>

        <div className="section-label" style={{ marginTop: "16px" }}>我的群聊</div>
        <div className="peer-list">
          {groups.length === 0 ? (
            <div className="empty-state" style={{ padding: "12px" }}>
              <p style={{ fontSize: "13px", opacity: 0.5 }}>暂无群聊</p>
            </div>
          ) : (
            groups.map((group) => (
              <div
                key={group.id}
                className={`peer-item ${selectedGroup === group.id ? "selected" : ""}`}
                onClick={() => {
                  setSelectedGroup(group.id);
                  setChatMode("group");
                  setSelectedPeer(null);
                }}
              >
                <div
                  className="avatar"
                  style={{ background: getAvatarColor(group.name) }}
                >
                  {group.name[0]}
                </div>
                <div className="peer-info">
                  <span className="peer-name">{group.name}</span>
                  <span className="peer-status">{group.member_count} 人</span>
                </div>
              </div>
            ))
          )}
        </div>

        <div className="group-actions" style={{ padding: "12px", display: "flex", gap: "8px" }}>
          <button
            className="btn-secondary"
            style={{ flex: 1, fontSize: "13px" }}
            onClick={() => setShowCreateGroup(true)}
          >
            创建群聊
          </button>
          <button
            className="btn-secondary"
            style={{ flex: 1, fontSize: "13px" }}
            onClick={() => setShowJoinGroup(true)}
          >
            加入群聊
          </button>
        </div>
      </div>

      <div className="chat-area">
        {selectedPeer ? (
          <>
            <div className="chat-header">
              <div className="chat-user">
                <div
                  className="avatar-sm"
                  style={{
                    background: getAvatarColor(
                      peers.find((p) => p.peer_id === selectedPeer)?.name || ""
                    ),
                  }}
                >
                  {peers.find((p) => p.peer_id === selectedPeer)?.name?.[0]}
                </div>
                <div>
                  <h3>
                    {peers.find((p) => p.peer_id === selectedPeer)?.name}
                  </h3>
                  <span className="status-text">在线</span>
                </div>
              </div>
            </div>

            <div className="messages">
              {messages.map((msg) => (
                <div
                  key={msg.id}
                  className={`message ${msg.from_peer === myPeerId ? "sent" : "received"}`}
                >
                  <div className="message-content">{msg.content}</div>
                  <div className="message-time">{formatTime(msg.timestamp)}</div>
                </div>
              ))}
            </div>

            <div className="input-area">
              <button className="icon-btn" onClick={handleFileSelect} title="发送文件">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"></path>
                </svg>
              </button>
              <input
                type="text"
                placeholder="输入消息..."
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSend()}
              />
              <button className="send-btn" onClick={handleSend}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="22" y1="2" x2="11" y2="13"></line>
                  <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
                </svg>
              </button>
            </div>
          </>
        ) : (
          <div className="no-chat">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" opacity="0.2">
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
            </svg>
            <p>选择一个设备开始聊天</p>
          </div>
        )}
      </div>

      {showSettings && (
        <div className="modal-overlay" onClick={() => setShowSettings(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>设置</h3>
            <div className="modal-content">
              <label>昵称</label>
              <input
                type="text"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                placeholder="输入新昵称"
              />
            </div>
            <div className="modal-actions">
              <button className="btn-secondary" onClick={() => setShowSettings(false)}>
                取消
              </button>
              <button className="btn-primary" onClick={handleUpdateName}>
                保存
              </button>
            </div>
          </div>
        </div>
      )}

      {showCreateGroup && (
        <div className="modal-overlay" onClick={() => { setShowCreateGroup(false); setCreatedPasscode(null); }}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>{createdPasscode ? "群聊已创建" : "创建群聊"}</h3>
            {!createdPasscode ? (
              <>
                <div className="modal-content">
                  <label>群名称</label>
                  <input
                    type="text"
                    value={newGroupName}
                    onChange={(e) => setNewGroupName(e.target.value)}
                    placeholder="输入群名称"
                    onKeyDown={(e) => e.key === "Enter" && handleCreateGroup()}
                  />
                </div>
                <div className="modal-actions">
                  <button className="btn-secondary" onClick={() => setShowCreateGroup(false)}>
                    取消
                  </button>
                  <button className="btn-primary" onClick={handleCreateGroup}>
                    创建
                  </button>
                </div>
              </>
            ) : (
              <>
                <div className="modal-content" style={{ textAlign: "center" }}>
                  <p style={{ marginBottom: "12px" }}>分享此口令邀请好友加入：</p>
                  <div
                    style={{
                      fontSize: "36px",
                      fontWeight: "bold",
                      letterSpacing: "8px",
                      color: "#10B981",
                      padding: "16px",
                      background: "rgba(16, 185, 129, 0.1)",
                      borderRadius: "8px",
                    }}
                  >
                    {createdPasscode}
                  </div>
                  <button
                    className="btn-secondary"
                    style={{ marginTop: "12px", fontSize: "13px" }}
                    onClick={() => {
                      navigator.clipboard.writeText(createdPasscode);
                    }}
                  >
                    复制口令
                  </button>
                </div>
                <div className="modal-actions">
                  <button
                    className="btn-primary"
                    onClick={() => {
                      setShowCreateGroup(false);
                      setCreatedPasscode(null);
                    }}
                  >
                    完成
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      )}

      {showJoinGroup && (
        <div className="modal-overlay" onClick={() => setShowJoinGroup(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>加入群聊</h3>
            <div className="modal-content">
              <label>输入 4 位口令</label>
              <input
                type="text"
                value={joinPasscode}
                onChange={(e) => {
                  const val = e.target.value.replace(/\D/g, "").slice(0, 4);
                  setJoinPasscode(val);
                }}
                placeholder="例如：5823"
                maxLength={4}
                onKeyDown={(e) => e.key === "Enter" && handleJoinGroup()}
              />
            </div>
            <div className="modal-actions">
              <button className="btn-secondary" onClick={() => setShowJoinGroup(false)}>
                取消
              </button>
              <button
                className="btn-primary"
                onClick={handleJoinGroup}
                disabled={joinPasscode.length !== 4}
              >
                加入
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
