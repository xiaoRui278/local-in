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

interface ChatHistoryItem {
  peer_id: string;
  peer_name: string;
  last_message: string;
  last_message_time: number;
  type: "private" | "group";
  group_id?: string;
  member_count?: number;
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
  const [fontFamily, setFontFamily] = useState(() => {
    return localStorage.getItem('font-family') || 'jetbrains';
  });

  const [groups, setGroups] = useState<GroupInfo[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [groupMessages, setGroupMessages] = useState<GroupMessageRecord[]>([]);
  const [showCreateGroup, setShowCreateGroup] = useState(false);
  const [showJoinGroup, setShowJoinGroup] = useState(false);
  const [newGroupName, setNewGroupName] = useState("");
  const [joinPasscode, setJoinPasscode] = useState("");
  const [createdPasscode, setCreatedPasscode] = useState<string | null>(null);
  const [chatMode, setChatMode] = useState<"global" | "group">("global");
  const [chatHistory, setChatHistory] = useState<ChatHistoryItem[]>([]);
  const [showMembers, setShowMembers] = useState(true);
  const [fontSize, setFontSize] = useState(() => {
    return localStorage.getItem('font-size') || '14';
  });

  useEffect(() => {
    loadSavedConfig();
  }, []);

  useEffect(() => {
    document.body.className = theme;
  }, [theme]);

  useEffect(() => {
    localStorage.setItem('font-family', fontFamily);
    document.documentElement.style.setProperty(
      '--font-family',
      fontFamily === 'jetbrains'
        ? "'JetBrains Mono', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
        : "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    );
  }, [fontFamily]);

  useEffect(() => {
    localStorage.setItem('font-size', fontSize);
    document.documentElement.style.setProperty('--font-size', `${fontSize}px`);
  }, [fontSize]);

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
    if (groups.length > 0) {
      setChatHistory((prev) => {
        const existingGroupIds = prev
          .filter((item) => item.type === "group")
          .map((item) => item.group_id);

        const newGroups = groups.filter(
          (g) => !existingGroupIds.includes(g.id)
        );

        if (newGroups.length === 0) return prev;

        const newItems: ChatHistoryItem[] = newGroups.map((group) => ({
          peer_id: group.id,
          peer_name: group.name,
          last_message: "",
          last_message_time: 0,
          type: "group" as const,
          group_id: group.id,
          member_count: group.member_count,
        }));

        return [...prev, ...newItems].sort(
          (a, b) => b.last_message_time - a.last_message_time
        );
      });
    }
  }, [groups]);

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

      setChatHistory((prev) => {
        const existing = prev.find(
          (item) => item.type === "private" && item.peer_id === selectedPeer
        );
        if (existing) {
          return prev
            .map((item) =>
              item.type === "private" && item.peer_id === selectedPeer
                ? {
                    ...item,
                    last_message: input.trim(),
                    last_message_time: Math.floor(Date.now() / 1000),
                  }
                : item
            )
            .sort((a, b) => b.last_message_time - a.last_message_time);
        } else {
          const peer = peers.find((p) => p.peer_id === selectedPeer);
          const newItem: ChatHistoryItem = {
            peer_id: selectedPeer!,
            peer_name: peer?.name || "Unknown",
            last_message: input.trim(),
            last_message_time: Math.floor(Date.now() / 1000),
            type: "private",
          };
          return [newItem, ...prev].sort(
            (a, b) => b.last_message_time - a.last_message_time
          );
        }
      });

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

      setChatHistory((prev) => {
        const group = groups.find((g) => g.id === selectedGroup);
        const existing = prev.find(
          (item) => item.type === "group" && item.group_id === selectedGroup
        );
        if (existing) {
          return prev
            .map((item) =>
              item.type === "group" && item.group_id === selectedGroup
                ? {
                    ...item,
                    last_message: input.trim(),
                    last_message_time: Math.floor(Date.now() / 1000),
                  }
                : item
            )
            .sort((a, b) => b.last_message_time - a.last_message_time);
        } else {
          const newItem: ChatHistoryItem = {
            peer_id: selectedGroup,
            peer_name: group?.name || "Unknown",
            last_message: input.trim(),
            last_message_time: Math.floor(Date.now() / 1000),
            type: "group",
            group_id: selectedGroup,
            member_count: group?.member_count,
          };
          return [newItem, ...prev].sort(
            (a, b) => b.last_message_time - a.last_message_time
          );
        }
      });

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

        <div className="sidebar-section sidebar-fixed">
          <div
            className={`peer-item ${chatMode === "global" && !selectedPeer ? "selected" : ""}`}
            onClick={() => {
              setChatMode("global");
              setSelectedGroup(null);
              setSelectedPeer(null);
            }}
          >
            <div className="avatar" style={{ background: "linear-gradient(135deg, #06B6D4, #0EA5E9)", display: "flex", alignItems: "center", justifyContent: "center" }}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2">
                <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"></path>
                <path d="M13.73 21a2 2 0 0 1-3.46 0"></path>
              </svg>
            </div>
            <div className="peer-info">
              <span className="peer-name">公共频道</span>
              <span className="peer-status">{peers.length} 在线</span>
            </div>
          </div>
        </div>

        <div className="section-label">我的聊天</div>
        <div className="chat-list">
          {chatHistory.length === 0 ? (
            <div className="empty-state" style={{ padding: "12px" }}>
              <p style={{ fontSize: "13px", opacity: 0.5 }}>暂无聊天记录</p>
            </div>
          ) : (
            chatHistory.map((item) => (
              <div
                key={item.type === "group" ? item.group_id : item.peer_id}
                className={`peer-item ${
                  (item.type === "group" && selectedGroup === item.group_id) ||
                  (item.type === "private" && selectedPeer === item.peer_id && chatMode === "global")
                    ? "selected" : ""
                }`}
                onClick={() => {
                  if (item.type === "group") {
                    setSelectedGroup(item.group_id!);
                    setChatMode("group");
                    setSelectedPeer(null);
                  } else {
                    setSelectedPeer(item.peer_id);
                    setChatMode("global");
                    setSelectedGroup(null);
                  }
                }}
              >
                <div className="avatar" style={{ background: getAvatarColor(item.peer_name) }}>
                  {item.peer_name[0]}
                </div>
                <div className="peer-info">
                  <span className="peer-name">
                    {item.type === "group" ? item.peer_name : item.peer_name}
                  </span>
                  <span className="peer-status">
                    {item.type === "group" ? `${item.member_count} 人` : item.last_message || "新消息"}
                  </span>
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
        {chatMode === "group" && selectedGroup ? (
          <>
            <div className="chat-header">
              <div className="chat-user">
                <div
                  className="avatar-sm"
                  style={{
                    background: getAvatarColor(
                      groups.find((g) => g.id === selectedGroup)?.name || ""
                    ),
                  }}
                >
                  {groups.find((g) => g.id === selectedGroup)?.name?.[0]}
                </div>
                <div>
                  <h3>{groups.find((g) => g.id === selectedGroup)?.name}</h3>
                  <span className="status-text">
                    {groups.find((g) => g.id === selectedGroup)?.member_count} 人
                  </span>
                </div>
              </div>
              <div className="header-actions">
                <button
                  className="icon-btn"
                  onClick={() => setShowMembers(!showMembers)}
                  title={showMembers ? "隐藏成员" : "显示成员"}
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                    <circle cx="9" cy="7" r="4"></circle>
                    <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
                    <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
                  </svg>
                </button>
                {groups.find((g) => g.id === selectedGroup)?.creator_peer === myPeerId ? (
                  <button
                    className="icon-btn"
                    onClick={handleDissolveGroup}
                    title="解散群聊"
                    style={{ color: "#EF4444" }}
                  >
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <circle cx="12" cy="12" r="10"></circle>
                      <line x1="15" y1="9" x2="9" y2="15"></line>
                      <line x1="9" y1="9" x2="15" y2="15"></line>
                    </svg>
                  </button>
                ) : (
                  <button
                    className="icon-btn"
                    onClick={handleLeaveGroup}
                    title="退出群聊"
                    style={{ color: "#F59E0B" }}
                  >
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path>
                      <polyline points="16 17 21 12 16 7"></polyline>
                      <line x1="21" y1="12" x2="9" y2="12"></line>
                    </svg>
                  </button>
                )}
              </div>
            </div>

            <div className="messages">
              {groupMessages.map((msg) => (
                <div
                  key={msg.id}
                  className={`message ${msg.from_peer === myPeerId ? "sent" : "received"}`}
                >
                  {msg.from_peer !== myPeerId && (
                    <div className="message-sender">{msg.from_name}</div>
                  )}
                  <div className="message-content">{msg.content}</div>
                  <div className="message-time">{formatTime(msg.timestamp)}</div>
                </div>
              ))}
            </div>

            <div className="input-area">
              <input
                type="text"
                placeholder="输入消息..."
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleSendGroupMessage()}
              />
              <button className="send-btn" onClick={handleSendGroupMessage}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <line x1="22" y1="2" x2="11" y2="13"></line>
                  <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
                </svg>
              </button>
            </div>
          </>
        ) : selectedPeer ? (
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
              <div className="header-actions">
                <button
                  className="icon-btn"
                  onClick={() => setShowMembers(!showMembers)}
                  title={showMembers ? "隐藏成员" : "显示成员"}
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
                    <circle cx="9" cy="7" r="4"></circle>
                    <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
                    <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
                  </svg>
                </button>
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
            <p>选择聊天或群组开始对话</p>
          </div>
        )}
      </div>

      {showMembers ? (
        <div className="members-sidebar">
          <div className="members-header">
            <h3>{chatMode === "group" ? "群成员" : "在线成员"}</h3>
            <button className="icon-btn" onClick={() => setShowMembers(false)} title="折叠">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="15 18 9 12 15 6"></polyline>
              </svg>
            </button>
          </div>
          <div className="members-list">
            {chatMode === "group" && selectedGroup ? (
              <div className="empty-state">
                <p>群成员列表</p>
              </div>
            ) : (
              peers.map((peer) => (
                <div
                  key={peer.peer_id}
                  className="member-item"
                  onClick={() => {
                    setSelectedPeer(peer.peer_id);
                    setChatMode("global");
                    setSelectedGroup(null);
                  }}
                >
                  <div className="avatar-sm" style={{ background: getAvatarColor(peer.name) }}>
                    {peer.name[0]}
                  </div>
                  <div className="member-info">
                    <span className="member-name">{peer.name}</span>
                    <span className="member-status">{peer.online ? "在线" : "离线"}</span>
                  </div>
                  <button
                    className="member-action"
                    title="发送消息"
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedPeer(peer.peer_id);
                      setChatMode("global");
                      setSelectedGroup(null);
                    }}
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
                    </svg>
                  </button>
                </div>
              ))
            )}
          </div>
        </div>
      ) : (
        <div className="members-sidebar-collapsed" onClick={() => setShowMembers(true)}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
            <circle cx="9" cy="7" r="4"></circle>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
          </svg>
          <span>{peers.length}</span>
        </div>
      )}

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

              <label style={{ marginTop: "12px" }}>字体</label>
              <select
                value={fontFamily}
                onChange={(e) => setFontFamily(e.target.value)}
                style={{
                  width: "100%",
                  padding: "8px 12px",
                  borderRadius: "6px",
                  border: "1px solid var(--border-color)",
                  background: "var(--bg-secondary)",
                  color: "var(--text-primary)",
                }}
              >
                <option value="jetbrains">JetBrains Mono</option>
                <option value="system">系统字体</option>
              </select>

              <label style={{ marginTop: "12px" }}>字体大小</label>
              <select
                value={fontSize}
                onChange={(e) => setFontSize(e.target.value)}
                style={{
                  width: "100%",
                  padding: "8px 12px",
                  borderRadius: "6px",
                  border: "1px solid var(--border-color)",
                  background: "var(--bg-secondary)",
                  color: "var(--text-primary)",
                }}
              >
                <option value="12">小 (12px)</option>
                <option value="14">中 (14px)</option>
                <option value="16">大 (16px)</option>
                <option value="18">特大 (18px)</option>
              </select>
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
