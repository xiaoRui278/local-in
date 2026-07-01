import type { Peer, GroupInfo, ChatMode } from "../../types";
import { UsersIcon, CloseIcon, LogoutIcon, BellIcon } from "../Icons";

interface ChatHeaderProps {
  chatMode: ChatMode;
  selectedPeer: string | null;
  selectedGroup: string | null;
  peers: Peer[];
  groups: GroupInfo[];
  myPeerId: string;
  showMembers: boolean;
  onToggleMembers: () => void;
  onDissolveGroup: () => void;
  onLeaveGroup: () => void;
  getAvatarColor: (name: string) => string;
}

export function ChatHeader({
  chatMode, selectedPeer, selectedGroup, peers, groups, myPeerId,
  showMembers, onToggleMembers, onDissolveGroup, onLeaveGroup, getAvatarColor,
}: ChatHeaderProps) {
  if (chatMode === "group" && selectedGroup) {
    const group = groups.find((g) => g.id === selectedGroup);
    const isCreator = group?.creator_peer === myPeerId;

    return (
      <header className="chat-header">
        <div className="chat-user">
          <div
            className="avatar-sm"
            style={{ background: getAvatarColor(group?.name || "") }}
            aria-hidden="true"
          >
            {group?.name?.[0]}
          </div>
          <div>
            <h3>{group?.name}</h3>
            <span className="status-text">{group?.member_count} 人</span>
          </div>
        </div>
        <div className="header-actions">
          <button
            className="icon-btn"
            onClick={onToggleMembers}
            aria-label={showMembers ? "隐藏成员" : "显示成员"}
            aria-pressed={showMembers}
          >
            <UsersIcon />
          </button>
          {isCreator ? (
            <button
              className="icon-btn"
              onClick={onDissolveGroup}
              aria-label="解散群聊"
              style={{ color: "var(--color-accent-red)" }}
            >
              <CloseIcon />
            </button>
          ) : (
            <button
              className="icon-btn"
              onClick={onLeaveGroup}
              aria-label="退出群聊"
              style={{ color: "var(--color-accent-amber)" }}
            >
              <LogoutIcon />
            </button>
          )}
        </div>
      </header>
    );
  }

  if (selectedPeer) {
    const peer = peers.find((p) => p.peer_id === selectedPeer);
    return (
      <header className="chat-header">
        <div className="chat-user">
          <div
            className="avatar-sm"
            style={{ background: getAvatarColor(peer?.name || "") }}
            aria-hidden="true"
          >
            {peer?.name?.[0]}
          </div>
          <div>
            <h3>{peer?.name}</h3>
            <span className="status-text">在线</span>
          </div>
        </div>
        <div className="header-actions">
          <button
            className="icon-btn"
            onClick={onToggleMembers}
            aria-label={showMembers ? "隐藏成员" : "显示成员"}
            aria-pressed={showMembers}
          >
            <UsersIcon />
          </button>
        </div>
      </header>
    );
  }

  if (chatMode === "global") {
    return (
      <header className="chat-header">
        <div className="chat-user">
          <div className="avatar-sm avatar-bell" aria-hidden="true">
            <BellIcon width={16} height={16} color="white" />
          </div>
          <div>
            <h3>公共频道</h3>
            <span className="status-text">{peers.length} 在线</span>
          </div>
        </div>
        <div className="header-actions">
          <button
            className="icon-btn"
            onClick={onToggleMembers}
            aria-label={showMembers ? "隐藏成员" : "显示成员"}
            aria-pressed={showMembers}
          >
            <UsersIcon />
          </button>
        </div>
      </header>
    );
  }

  return null;
}
