import type { Peer, ChatMode } from "../types";
import { UsersIcon, ChevronLeftIcon, ChatIcon } from "./Icons";

interface MembersPanelProps {
  show: boolean;
  chatMode: ChatMode;
  peers: Peer[];
  onToggle: () => void;
  onSelectPeer: (peerId: string) => void;
  getAvatarColor: (name: string) => string;
}

export function MembersPanel({
  show, chatMode, peers, onToggle, onSelectPeer, getAvatarColor,
}: MembersPanelProps) {
  if (!show) {
    return (
      <div className="members-sidebar-collapsed" onClick={onToggle} role="button" tabIndex={0} aria-label="展开成员列表">
        <UsersIcon width={16} height={16} />
        <span>{peers.length}</span>
      </div>
    );
  }

  return (
    <div className="members-sidebar" role="complementary" aria-label="成员列表">
      <div className="members-header">
        <h3>{chatMode === "group" ? "群成员" : "在线成员"}</h3>
        <button className="icon-btn" onClick={onToggle} aria-label="折叠成员列表">
          <ChevronLeftIcon width={16} height={16} />
        </button>
      </div>
      <div className="members-list">
        {chatMode === "group" ? (
          <div className="empty-state">
            <p>群成员列表</p>
          </div>
        ) : (
          peers.map((peer) => (
            <div
              key={peer.peer_id}
              className="member-item"
              onClick={() => onSelectPeer(peer.peer_id)}
              role="button"
              tabIndex={0}
              onKeyDown={(e) => e.key === "Enter" && onSelectPeer(peer.peer_id)}
            >
              <div className="avatar-sm" style={{ background: getAvatarColor(peer.name) }} aria-hidden="true">
                {peer.name[0]}
              </div>
              <div className="member-info">
                <span className="member-name">{peer.name}</span>
                <span className="member-status">{peer.online ? "在线" : "离线"}</span>
              </div>
              <button
                className="member-action"
                aria-label={`给 ${peer.name} 发送消息`}
                onClick={(e) => {
                  e.stopPropagation();
                  onSelectPeer(peer.peer_id);
                }}
              >
                <ChatIcon width={14} height={14} />
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
