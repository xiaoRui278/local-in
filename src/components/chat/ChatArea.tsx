import type { Peer, MessageRecord, GroupMessageRecord, GroupInfo, ChatMode } from "../../types";
import { ChatHeader } from "./ChatHeader";
import { MessageBubble } from "./MessageBubble";
import { FileCard } from "./FileCard";
import { InputArea } from "./InputArea";
import { ChatLargeIcon } from "../Icons";

interface ChatAreaProps {
  chatMode: ChatMode;
  selectedPeer: string | null;
  selectedGroup: string | null;
  peers: Peer[];
  groups: GroupInfo[];
  myPeerId: string;
  messages: MessageRecord[];
  globalMessages: MessageRecord[];
  groupMessages: GroupMessageRecord[];
  input: string;
  showMembers: boolean;
  globalMessagesRef: React.RefObject<HTMLDivElement | null>;
  privateMessagesRef: React.RefObject<HTMLDivElement | null>;
  onInputChange: (value: string) => void;
  onSend: () => void;
  onSendGlobal: () => void;
  onSendGroup: () => void;
  onFileSelect: () => void;
  onToggleMembers: () => void;
  onDissolveGroup: () => void;
  onLeaveGroup: () => void;
  onAcceptFile: (fileId: string, fromPeer: string, messageId: string) => void;
  onRejectFile: (fileId: string, fromPeer: string, messageId: string) => void;
  onCancelFileTransfer: (fileId: string) => void;
  onRetryFileTransfer: (fileId: string) => void;
  formatTime: (timestamp: number) => string;
  getAvatarColor: (name: string) => string;
}

export function ChatArea({
  chatMode, selectedPeer, selectedGroup, peers, groups, myPeerId,
  messages, globalMessages, groupMessages, input, showMembers,
  globalMessagesRef, privateMessagesRef,
  onInputChange, onSend, onSendGlobal, onSendGroup, onFileSelect,
  onToggleMembers, onDissolveGroup, onLeaveGroup, onAcceptFile, onRejectFile,
  onCancelFileTransfer, onRetryFileTransfer,
  formatTime, getAvatarColor,
}: ChatAreaProps) {
  const renderMessages = (msgs: MessageRecord[], isGlobal: boolean) => (
    <div className="messages" ref={isGlobal ? globalMessagesRef as React.RefObject<HTMLDivElement> : privateMessagesRef as React.RefObject<HTMLDivElement>}>
      {msgs.slice(-50).map((msg) => (
        msg.content.startsWith("[FILE]") ? (
          <FileCard
            key={msg.id}
            message={msg}
            isMine={msg.from_peer === myPeerId}
            onAccept={onAcceptFile}
            onReject={onRejectFile}
            onCancel={onCancelFileTransfer}
            onRetry={onRetryFileTransfer}
          />
        ) : (
          <MessageBubble
            key={msg.id}
            message={msg}
            isMine={msg.from_peer === myPeerId}
            showSender={isGlobal && msg.from_peer !== myPeerId}
            formatTime={formatTime}
          />
        )
      ))}
    </div>
  );

  const renderGroupMessages = () => (
    <div className="messages">
      {groupMessages.slice(-50).map((msg) => (
        <MessageBubble
          key={msg.id}
          message={{ ...msg, is_read: true, to_peer: selectedGroup || "" }}
          isMine={msg.from_peer === myPeerId}
          showSender={msg.from_peer !== myPeerId}
          formatTime={formatTime}
        />
      ))}
    </div>
  );

  if (chatMode === "group" && selectedGroup) {
    return (
      <div className="chat-area">
        <ChatHeader
          chatMode={chatMode}
          selectedPeer={selectedPeer}
          selectedGroup={selectedGroup}
          peers={peers}
          groups={groups}
          myPeerId={myPeerId}
          showMembers={showMembers}
          onToggleMembers={onToggleMembers}
          onDissolveGroup={onDissolveGroup}
          onLeaveGroup={onLeaveGroup}
          getAvatarColor={getAvatarColor}
        />
        {renderGroupMessages()}
        <InputArea
          value={input}
          onChange={onInputChange}
          onSend={onSendGroup}
        />
      </div>
    );
  }

  if (selectedPeer) {
    return (
      <div className="chat-area">
        <ChatHeader
          chatMode={chatMode}
          selectedPeer={selectedPeer}
          selectedGroup={selectedGroup}
          peers={peers}
          groups={groups}
          myPeerId={myPeerId}
          showMembers={showMembers}
          onToggleMembers={onToggleMembers}
          onDissolveGroup={onDissolveGroup}
          onLeaveGroup={onLeaveGroup}
          getAvatarColor={getAvatarColor}
        />
        {renderMessages(messages, false)}
        <InputArea
          value={input}
          onChange={onInputChange}
          onSend={onSend}
          onFileSelect={onFileSelect}
          showFileButton={true}
        />
      </div>
    );
  }

  if (chatMode === "global") {
    return (
      <div className="chat-area">
        <ChatHeader
          chatMode={chatMode}
          selectedPeer={selectedPeer}
          selectedGroup={selectedGroup}
          peers={peers}
          groups={groups}
          myPeerId={myPeerId}
          showMembers={showMembers}
          onToggleMembers={onToggleMembers}
          onDissolveGroup={onDissolveGroup}
          onLeaveGroup={onLeaveGroup}
          getAvatarColor={getAvatarColor}
        />
        {renderMessages(globalMessages, true)}
        <InputArea
          value={input}
          onChange={onInputChange}
          onSend={onSendGlobal}
        />
      </div>
    );
  }

  return (
    <div className="chat-area">
      <div className="no-chat">
        <ChatLargeIcon width={64} height={64} />
        <p>选择聊天或群组开始对话</p>
      </div>
    </div>
  );
}
