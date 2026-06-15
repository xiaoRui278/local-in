import { useState, useEffect } from "react";
import type { Theme, FontFamilyOption, FontSizeOption } from "./types";
import { LoginScreen } from "./components/LoginScreen";
import { Sidebar } from "./components/Sidebar";
import { ChatArea } from "./components/chat/ChatArea";
import { MembersPanel } from "./components/MembersPanel";
import { SettingsModal } from "./components/modals/SettingsModal";
import { CreateGroupModal } from "./components/modals/CreateGroupModal";
import { JoinGroupModal } from "./components/modals/JoinGroupModal";
import { useChat } from "./hooks/useChat";

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

function formatTime(timestamp: number) {
  const date = new Date(timestamp * 1000);
  return date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
}

function App() {
  const [theme, setTheme] = useState<Theme>("dark");
  const [showSettings, setShowSettings] = useState(false);
  const [showCreateGroup, setShowCreateGroup] = useState(false);
  const [showJoinGroup, setShowJoinGroup] = useState(false);
  const [showMembers, setShowMembers] = useState(true);
  const [fontFamily, setFontFamily] = useState<FontFamilyOption>(() => {
    return (localStorage.getItem("font-family") as FontFamilyOption) || "jetbrains";
  });
  const [fontSize, setFontSize] = useState<FontSizeOption>(() => {
    return (localStorage.getItem("font-size") as FontSizeOption) || "14";
  });

  const chat = useChat();

  useEffect(() => {
    document.body.className = theme;
  }, [theme]);

  useEffect(() => {
    localStorage.setItem("font-family", fontFamily);
    document.documentElement.style.setProperty(
      "--font-family",
      fontFamily === "jetbrains"
        ? "'JetBrains Mono', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
        : "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    );
  }, [fontFamily]);

  useEffect(() => {
    localStorage.setItem("font-size", fontSize);
    document.documentElement.style.setProperty("--font-size", `${fontSize}px`);
  }, [fontSize]);

  const handleToggleTheme = () => {
    setTheme((prev) => (prev === "dark" ? "light" : "dark"));
  };

  const handleSaveSettings = (name: string, font: FontFamilyOption, size: FontSizeOption) => {
    chat.handleUpdateName(name);
    setFontFamily(font);
    setFontSize(size);
    setShowSettings(false);
  };

  const handleSelectGlobal = () => {
    chat.setChatMode("global");
    chat.setSelectedGroup(null);
    chat.setSelectedPeer(null);
  };

  const handleSelectPrivate = (peerId: string) => {
    chat.setSelectedPeer(peerId);
    chat.setChatMode("global");
    chat.setSelectedGroup(null);
  };

  const handleSelectGroup = (groupId: string) => {
    chat.setSelectedGroup(groupId);
    chat.setChatMode("group");
    chat.setSelectedPeer(null);
  };

  const handleJoinGroup = async (passcode: string) => {
    const result = await chat.handleJoinGroup(passcode);
    return result !== null;
  };

  if (!chat.started) {
    return (
      <div className={`app ${theme}`}>
        <LoginScreen
          theme={theme}
          onToggleTheme={handleToggleTheme}
          onStart={chat.handleStart}
        />
      </div>
    );
  }

  return (
    <div className={`app ${theme}`}>
      <Sidebar
        theme={theme}
        name={chat.name}
        peers={chat.peers}
        chatHistory={chat.chatHistory}
        chatMode={chat.chatMode}
        selectedPeer={chat.selectedPeer}
        selectedGroup={chat.selectedGroup}
        onToggleTheme={handleToggleTheme}
        onOpenSettings={() => setShowSettings(true)}
        onSelectGlobal={handleSelectGlobal}
        onSelectPrivate={handleSelectPrivate}
        onSelectGroup={handleSelectGroup}
        onCreateGroup={() => setShowCreateGroup(true)}
        onJoinGroup={() => setShowJoinGroup(true)}
      />

      <ChatArea
        chatMode={chat.chatMode}
        selectedPeer={chat.selectedPeer}
        selectedGroup={chat.selectedGroup}
        peers={chat.peers}
        groups={chat.groups}
        myPeerId={chat.myPeerId}
        messages={chat.messages}
        globalMessages={chat.globalMessages}
        groupMessages={chat.groupMessages}
        input={chat.input}
        showMembers={showMembers}
        globalMessagesRef={chat.globalMessagesRef}
        privateMessagesRef={chat.privateMessagesRef}
        onInputChange={chat.setInput}
        onSend={chat.handleSend}
        onSendGlobal={chat.handleSendGlobalMessage}
        onSendGroup={chat.handleSendGroupMessage}
        onFileSelect={chat.handleFileSelect}
        onToggleMembers={() => setShowMembers(!showMembers)}
        onDissolveGroup={chat.handleDissolveGroup}
        onLeaveGroup={chat.handleLeaveGroup}
        onAcceptFile={chat.handleAcceptFile}
        formatTime={formatTime}
        getAvatarColor={getAvatarColor}
      />

      <MembersPanel
        show={showMembers}
        chatMode={chat.chatMode}
        peers={chat.peers}
        onToggle={() => setShowMembers(!showMembers)}
        onSelectPeer={handleSelectPrivate}
        getAvatarColor={getAvatarColor}
      />

      <SettingsModal
        show={showSettings}
        theme={theme}
        currentName={chat.name}
        currentFont={fontFamily}
        currentSize={fontSize}
        onClose={() => setShowSettings(false)}
        onSave={handleSaveSettings}
        onToggleTheme={handleToggleTheme}
      />

      <CreateGroupModal
        show={showCreateGroup}
        onClose={() => setShowCreateGroup(false)}
        onCreate={chat.handleCreateGroup}
      />

      <JoinGroupModal
        show={showJoinGroup}
        onClose={() => setShowJoinGroup(false)}
        onJoin={handleJoinGroup}
      />
    </div>
  );
}

export default App;
