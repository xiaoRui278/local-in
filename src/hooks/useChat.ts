import { useState, useEffect, useRef, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { Peer, MessageRecord, GroupInfo, GroupMessageRecord, ChatHistoryItem, ChatHistoryRecord, MessagePayload, FilePayload, GroupMember, GroupEventPayload, FileTransferEvent } from "../types";

export function useChat() {
  const [name, setName] = useState("");
  const [started, setStarted] = useState(false);
  const [peers, setPeers] = useState<Peer[]>([]);
  const [messages, setMessages] = useState<MessageRecord[]>([]);
  const [input, setInput] = useState("");
  const [selectedPeer, setSelectedPeer] = useState<string | null>(null);
  const [myPeerId, setMyPeerId] = useState("");
  const myPeerIdRef = useRef("");
  const [groups, setGroups] = useState<GroupInfo[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [groupMessages, setGroupMessages] = useState<GroupMessageRecord[]>([]);
  const [globalMessages, setGlobalMessages] = useState<MessageRecord[]>([]);
  const [chatMode, setChatMode] = useState<"global" | "group">("global");
  const [chatHistory, setChatHistory] = useState<ChatHistoryItem[]>([]);
  const [groupMembers, setGroupMembers] = useState<Record<string, GroupMember[]>>({});

  const globalMessagesRef = useRef<HTMLDivElement>(null);
  const privateMessagesRef = useRef<HTMLDivElement>(null);

  const loadSavedConfig = useCallback(async () => {
    try {
      const [savedName] = await invoke<[string | null, string | null]>("get_saved_config");
      if (savedName) setName(savedName);
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }, []);

  const loadMessages = useCallback(async (peerId: string) => {
    try {
      const msgs = await invoke<MessageRecord[]>("get_dm_messages", {
        peer1: myPeerId,
        peer2: peerId,
        limit: 100,
      });
      setMessages(msgs.reverse());
    } catch (e) {
      console.error("Failed to load messages:", e);
    }
  }, [myPeerId]);

  const loadGlobalMessages = useCallback(async () => {
    try {
      const msgs = await invoke<MessageRecord[]>("get_global_messages", { limit: 100 });
      setGlobalMessages(msgs.reverse());
    } catch (e) {
      console.error("Failed to load global messages:", e);
    }
  }, []);

  const loadGroups = useCallback(async () => {
    try {
      const groupList = await invoke<GroupInfo[]>("get_groups");
      setGroups(groupList);
    } catch (e) {
      console.error("Failed to get groups:", e);
    }
  }, []);

  const loadChatHistory = useCallback(async () => {
    try {
      const history = await invoke<ChatHistoryRecord[]>("get_chat_history");
      setChatHistory(history.map((item) => ({
        peer_id: item.peer_id,
        peer_name: item.peer_name,
        last_message: item.last_message,
        last_message_time: item.last_message_time,
        type: item.record_type,
        group_id: item.group_id || undefined,
        member_count: item.member_count || undefined,
      })));
    } catch (e) {
      console.error("Failed to load chat history:", e);
    }
  }, []);


  const loadGroupMessages = useCallback(async (groupId: string) => {
    try {
      const msgs = await invoke<GroupMessageRecord[]>("get_group_messages_cmd", {
        groupId,
        limit: 100,
      });
      setGroupMessages(msgs.reverse());
    } catch (e) {
      console.error("Failed to get group messages:", e);
    }
  }, []);

  const loadGroupMembers = useCallback(async (groupId: string) => {
    try {
      const members = await invoke<GroupMember[]>("get_group_members", { groupId });
      setGroupMembers((prev) => ({ ...prev, [groupId]: members }));
    } catch (e) {
      console.error("Failed to get group members:", e);
    }
  }, []);

  const handleStart = useCallback(async (startName: string) => {
    const trimmedName = startName.trim();
    if (!trimmedName) return;
    setName(trimmedName);
    try {
      const onMessage = new Channel<MessagePayload>();
      onMessage.onmessage = (payload) => {
        const msg = payload.record;

        if (msg.content.startsWith("[FILE]")) {
          const parts = msg.content.substring(6).split("|");
          if (parts.length >= 3) {
            const fileSize = parseInt(parts[2], 10) || 0;
            const fileMsg: MessageRecord = {
              ...msg,
              content: msg.content,
              file_id: parts[0],
              file_name: parts[1],
              file_size: fileSize,
              file_status: "pending",
              file_progress: 0,
              received_size: 0,
              transfer_speed: 0,
            };
            if (msg.to_peer === myPeerIdRef.current) {
              setMessages((prev) => [...prev, fileMsg]);
            }
          }
        } else {
          if (msg.to_peer === "global" || msg.to_peer === "") {
            setGlobalMessages((prev) => [...prev, msg]);
          } else if (msg.to_peer === myPeerIdRef.current || msg.from_peer === myPeerIdRef.current) {
            setMessages((prev) => [...prev, msg]);
          }
        }

        // Only private messages go to chatHistory here - group messages handled separately
        if (msg.to_peer !== "global" && msg.to_peer !== "" && !msg.to_peer.startsWith("group-")) {
          setChatHistory((prev) => {
            const peerId = msg.from_peer === myPeerIdRef.current ? msg.to_peer : msg.from_peer;
            const peerName = msg.from_name;

            const existing = prev.find((item) =>
              item.peer_id === peerId
            );

            if (existing) {
              return prev
                .map((item) =>
                  item.peer_id === peerId
                    ? { ...item, last_message: msg.content, last_message_time: msg.timestamp }
                    : item
                )
                .sort((a, b) => b.last_message_time - a.last_message_time);
            } else {
              const newItem: ChatHistoryItem = {
                peer_id: peerId,
                peer_name: peerName,
                last_message: msg.content,
                last_message_time: msg.timestamp,
                type: "private",
              };
              return [newItem, ...prev].sort((a, b) => b.last_message_time - a.last_message_time);
            }
          });
        }
      };

      const onFile = new Channel<FilePayload>();
      onFile.onmessage = (payload) => {
        console.log("Received file:", payload);
        setMessages(prev => prev.map(msg =>
          msg.file_id === payload.file_id && msg.from_peer === payload.from
            ? { ...msg, file_status: "completed" }
            : msg
        ));
      };

      const onGroupEvent = new Channel<GroupEventPayload>();
      onGroupEvent.onmessage = (payload) => {
        switch (payload.kind) {
          case "chat": {
            const newMessage: GroupMessageRecord = {
              id: Date.now().toString(),
              group_id: payload.group_id,
              from_peer: payload.from_peer,
              from_name: payload.from_name,
              content: payload.content,
              timestamp: payload.timestamp,
            };
            setGroupMessages((prev) => [...prev, newMessage]);
            setChatHistory((prev) => {
              const existing = prev.find(
                (item) => item.type === "group" && item.group_id === payload.group_id
              );
              if (existing) {
                return prev
                  .map((item) =>
                    item.type === "group" && item.group_id === payload.group_id
                      ? { ...item, last_message: payload.content, last_message_time: payload.timestamp }
                      : item
                  )
                  .sort((a, b) => b.last_message_time - a.last_message_time);
              }
              const newItem: ChatHistoryItem = {
                peer_id: payload.group_id,
                peer_name: payload.group_name,
                last_message: payload.content,
                last_message_time: payload.timestamp,
                type: "group",
                group_id: payload.group_id,
              };
              return [newItem, ...prev].sort((a, b) => b.last_message_time - a.last_message_time);
            });
            break;
          }
          case "join": {
            setGroupMembers((prev) => {
              const members = prev[payload.group_id] || [];
              const member: GroupMember = {
                group_id: payload.group_id,
                peer_id: payload.peer_id,
                peer_name: payload.peer_name,
                joined_at: payload.joined_at,
              };
              return {
                ...prev,
                [payload.group_id]: [
                  ...members.filter((item) => item.peer_id !== payload.peer_id),
                  member,
                ],
              };
            });
            loadGroupMembers(payload.group_id);
            break;
          }
          case "leave": {
            setGroupMembers((prev) => {
              const members = prev[payload.group_id] || [];
              return {
                ...prev,
                [payload.group_id]: members.filter(m => m.peer_id !== payload.peer_id),
              };
            });
            break;
          }
          case "dissolve": {
            setGroups((prev) => prev.filter((g) => g.id !== payload.group_id));
            setGroupMembers((prev) => {
              const next = { ...prev };
              delete next[payload.group_id];
              return next;
            });
            setChatHistory((prev) =>
              prev.filter((item) => !(item.type === "group" && item.group_id === payload.group_id))
            );
            setSelectedGroup((current) => {
              if (current === payload.group_id) {
                setChatMode("global");
                return null;
              }
              return current;
            });
            break;
          }
          case "sync": {
            setGroups((prev) => {
              const existing = prev.find((g) => g.id === payload.group_id);
              const groupInfo: GroupInfo = {
                id: payload.group_id,
                name: payload.group_name,
                passcode: payload.passcode,
                creator_peer: payload.creator_peer,
                member_count: payload.members.length,
              };
              if (existing) {
                return prev.map((g) => g.id === payload.group_id ? groupInfo : g);
              }
              return [groupInfo, ...prev];
            });
            const convertedMembers: GroupMember[] = payload.members.map((m) => ({
              group_id: payload.group_id,
              peer_id: m.peer_id,
              peer_name: m.peer_name,
              joined_at: m.joined_at,
            }));
            setGroupMembers((prev) => ({
              ...prev,
              [payload.group_id]: convertedMembers,
            }));
            break;
          }
        }
      };

      const onFileTransfer = new Channel<FileTransferEvent>();
      onFileTransfer.onmessage = (event) => {
        setMessages((prev) =>
          prev.map((msg) => {
            if (msg.file_id !== event.file_id) return msg;
            switch (event.kind) {
              case "progress":
                return {
                  ...msg,
                  file_status: event.status,
                  file_progress: event.total_size === 0 ? 0 : event.received_size / event.total_size,
                  received_size: event.received_size,
                  file_size: event.total_size,
                  transfer_speed: event.speed,
                  error_message: undefined,
                };
              case "completed":
                return {
                  ...msg,
                  file_status: "completed",
                  file_progress: 1,
                  received_size: msg.file_size || msg.received_size,
                  transfer_speed: 0,
                  file_path: event.file_path,
                };
              case "failed":
                return {
                  ...msg,
                  file_status: "failed",
                  transfer_speed: 0,
                  error_message: event.error_message,
                };
              case "cancelled":
                return {
                  ...msg,
                  file_status: "cancelled",
                  transfer_speed: 0,
                };
            }
          })
        );
      };

      const peerId = await invoke<string>("start_node", { name: trimmedName, onMessage, onFile, onGroupEvent, onFileTransfer });
      setMyPeerId(peerId);
      myPeerIdRef.current = peerId;
      setStarted(true);
    } catch (e) {
      console.error("Failed to start node:", e);
    }
  }, [loadGroupMembers]);

  const handleSend = useCallback(async () => {
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
          from_name: name,
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
                ? { ...item, last_message: input.trim(), last_message_time: Math.floor(Date.now() / 1000) }
                : item
            )
            .sort((a, b) => b.last_message_time - a.last_message_time);
        } else {
          const peer = peers.find((p) => p.peer_id === selectedPeer);
          const newItem: ChatHistoryItem = {
            peer_id: selectedPeer,
            peer_name: peer?.name || "Unknown",
            last_message: input.trim(),
            last_message_time: Math.floor(Date.now() / 1000),
            type: "private",
          };
          return [newItem, ...prev].sort((a, b) => b.last_message_time - a.last_message_time);
        }
      });

      setInput("");
    } catch (e) {
      console.error("Failed to send message:", e);
    }
  }, [input, selectedPeer, myPeerId, name, peers]);

  const handleSendGlobalMessage = useCallback(async () => {
    if (!input.trim()) return;
    try {
      await invoke("send_global_message", { from: myPeerId, content: input.trim() });
      setGlobalMessages((prev) => [
        ...prev,
        {
          id: Date.now().toString(),
          from_peer: myPeerId,
          from_name: name,
          to_peer: "global",
          content: input.trim(),
          timestamp: Math.floor(Date.now() / 1000),
          is_read: true,
        },
      ]);
      setInput("");
    } catch (e) {
      console.error("Failed to send global message:", e);
    }
  }, [input, myPeerId, name]);

  const handleSendGroupMessage = useCallback(async () => {
    if (!input.trim() || !selectedGroup) return;
    try {
      await invoke("send_group_message_cmd", { groupId: selectedGroup, content: input.trim() });
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
                ? { ...item, last_message: input.trim(), last_message_time: Math.floor(Date.now() / 1000) }
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
          return [newItem, ...prev].sort((a, b) => b.last_message_time - a.last_message_time);
        }
      });

      setInput("");
    } catch (e) {
      console.error("Failed to send group message:", e);
    }
  }, [input, selectedGroup, myPeerId, name, groups]);

  const handleCreateGroup = useCallback(async (newGroupName: string) => {
    if (!newGroupName.trim()) return null;
    try {
      const group = await invoke<GroupInfo>("create_group", { name: newGroupName.trim() });
      setGroups((prev) => [group, ...prev]);
      await loadGroupMembers(group.id);
      return group;
    } catch (e) {
      console.error("Failed to create group:", e);
      return null;
    }
  }, [loadGroupMembers]);

  const handleJoinGroup = useCallback(async (passcode: string) => {
    if (!passcode.trim() || passcode.length !== 4) return null;
    try {
      const group = await invoke<GroupInfo>("join_group", { passcode: passcode.trim() });
      setGroups((prev) => {
        const exists = prev.find((g) => g.id === group.id);
        if (exists) return prev;
        return [group, ...prev];
      });
      setSelectedGroup(group.id);
      setChatMode("group");
      await loadGroupMembers(group.id);
      return group;
    } catch (e) {
      console.error("Failed to join group:", e);
      return null;
    }
  }, [loadGroupMembers]);

  const handleDissolveGroup = useCallback(async () => {
    if (!selectedGroup) return;
    try {
      await invoke("dissolve_group", { groupId: selectedGroup });
      setGroups((prev) => prev.filter((g) => g.id !== selectedGroup));
      setSelectedGroup(null);
      setChatMode("global");
    } catch (e) {
      console.error("Failed to dissolve group:", e);
    }
  }, [selectedGroup]);

  const handleLeaveGroup = useCallback(async () => {
    if (!selectedGroup) return;
    try {
      await invoke("leave_group", { groupId: selectedGroup });
      setGroups((prev) => prev.filter((g) => g.id !== selectedGroup));
      setSelectedGroup(null);
      setChatMode("global");
    } catch (e) {
      console.error("Failed to leave group:", e);
    }
  }, [selectedGroup]);

  const handleUpdateName = useCallback(async (newName: string) => {
    if (!newName.trim()) return false;
    try {
      await invoke("update_name", { newName: newName.trim() });
      setName(newName.trim());
      return true;
    } catch (e) {
      console.error("Failed to update name:", e);
      return false;
    }
  }, []);

  const handleFileSelect = useCallback(async () => {
    if (!selectedPeer) return;
    const { open } = await import("@tauri-apps/plugin-dialog");
    try {
      const file = await open({ multiple: false });
      if (file) {
        const filePath = typeof file === "string" ? file : (file as { path: string }).path;
        try {
          const result = await invoke<string>("send_file", { peerId: selectedPeer, filePath });
          const stat = await invoke<{ size: number; name: string }>("get_file_stat", { filePath });
          const fileName = stat.name || filePath.split("/").pop() || filePath;
          const fileMsg: MessageRecord = {
            id: Date.now().toString(),
            from_peer: myPeerId,
            from_name: name,
            to_peer: selectedPeer,
            content: `[FILE]${result}|${fileName}|${stat.size}`,
            timestamp: Math.floor(Date.now() / 1000),
            is_read: true,
            file_id: result,
            file_name: fileName,
            file_size: stat.size,
            file_status: "pending",
            file_progress: 0,
            received_size: 0,
            transfer_speed: 0,
          };
          setMessages((prev) => [...prev, fileMsg]);
        } catch (e) {
          console.error("Failed to send file:", e);
          alert("发送失败: " + e);
        }
      }
    } catch (e) {
      console.error("Failed to open dialog:", e);
      alert("打开对话框失败: " + e);
    }
  }, [selectedPeer, myPeerId, name]);

  const handleAcceptFile = useCallback(async (fileId: string, fromPeer: string, messageId: string) => {
    try {
      await invoke("accept_file", { fileId, fromPeer });
      setMessages((prev) =>
        prev.map((m) => (m.id === messageId ? { ...m, file_status: "transferring", error_message: undefined } : m))
      );
    } catch (e) {
      console.error("Failed to accept file:", e);
    }
  }, []);

  const handleCancelFileTransfer = useCallback(async (fileId: string) => {
    try {
      await invoke("cancel_file_transfer", { fileId });
      setMessages((prev) =>
        prev.map((m) => (m.file_id === fileId ? { ...m, file_status: "cancelled", transfer_speed: 0 } : m))
      );
    } catch (e) {
      console.error("Failed to cancel file transfer:", e);
    }
  }, []);

  const handleRetryFileTransfer = useCallback(async (fileId: string) => {
    try {
      await invoke("retry_file_transfer", { fileId });
      setMessages((prev) =>
        prev.map((m) => (m.file_id === fileId ? { ...m, file_status: "transferring", error_message: undefined } : m))
      );
    } catch (e) {
      console.error("Failed to retry file transfer:", e);
    }
  }, []);

  useEffect(() => { loadSavedConfig(); }, [loadSavedConfig]);

  useEffect(() => {
    if (globalMessagesRef.current) {
      globalMessagesRef.current.scrollTop = globalMessagesRef.current.scrollHeight;
    }
  }, [globalMessages]);

  useEffect(() => {
    if (privateMessagesRef.current) {
      privateMessagesRef.current.scrollTop = privateMessagesRef.current.scrollHeight;
    }
  }, [messages, groupMessages]);

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
    if (selectedPeer) loadMessages(selectedPeer);
  }, [selectedPeer, loadMessages]);

  useEffect(() => {
    if (started) {
      loadGroups();
      loadGlobalMessages();
      loadChatHistory();
    }
  }, [started, loadGroups, loadGlobalMessages, loadChatHistory]);

  useEffect(() => {
    if (groups.length > 0) {
      setChatHistory((prev) => {
        const existingGroupIds = prev.filter((item) => item.type === "group").map((item) => item.group_id);
        const newGroups = groups.filter((g) => !existingGroupIds.includes(g.id));
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
        return [...prev, ...newItems].sort((a, b) => b.last_message_time - a.last_message_time);
      });
    }
  }, [groups]);

  useEffect(() => {
    if (selectedGroup && chatMode === "group") loadGroupMessages(selectedGroup);
  }, [selectedGroup, chatMode, loadGroupMessages]);

  useEffect(() => {
    if (selectedGroup && chatMode === "group") {
      loadGroupMembers(selectedGroup);
    }
  }, [selectedGroup, chatMode, loadGroupMembers]);

  return {
    name, setName,
    started,
    peers,
    messages,
    input, setInput,
    selectedPeer, setSelectedPeer,
    myPeerId,
    groups,
    selectedGroup, setSelectedGroup,
    groupMessages,
    groupMembers,
    loadGroupMembers,
    globalMessages,
    chatMode, setChatMode,
    chatHistory,
    globalMessagesRef,
    privateMessagesRef,
    handleStart,
    handleSend,
    handleSendGlobalMessage,
    handleSendGroupMessage,
    handleCreateGroup,
    handleJoinGroup,
    handleDissolveGroup,
    handleLeaveGroup,
    handleUpdateName,
    handleFileSelect,
    handleAcceptFile,
    handleCancelFileTransfer,
    handleRetryFileTransfer,
  };
}
