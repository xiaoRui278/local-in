package wang.xiaorui.local.server;

import io.libp2p.core.PeerId;
import wang.xiaorui.local.p2p.model.P2PUser;

import java.util.*;
import java.util.concurrent.ConcurrentHashMap;

/**
 * @author wangxiaorui
 * @date 2025/2/13
 * @desc 连接缓存
 */
public class ConnectionCache {
    private static volatile ConnectionCache instance;
    private static final List<ConnectionListener> connectionListeners = new ArrayList<>();

    private ConnectionCache() {
    }

    public static ConnectionCache getInstance() {
        if (instance == null) {
            synchronized (ConnectionCache.class) {
                if (instance == null) {
                    instance = new ConnectionCache();
                }
            }
        }
        return instance;
    }

    /**
     * 已知节点
     */
    private static final Set<PeerId> knownNodes = new HashSet<>();
    private static final Map<PeerId, LocalInUser> peers = new ConcurrentHashMap<>();

    public void addListener(ConnectionListener listener) {
        connectionListeners.add(listener);
    }

    public void addKnownNode(PeerId peerId) {
        knownNodes.add(peerId);
    }

    public boolean isKnownNode(PeerId peerId) {
        return knownNodes.contains(peerId);
    }

    public void removeKnownNode(PeerId peerId) {
        knownNodes.remove(peerId);
    }

    public void addPeer(PeerId peerId, LocalInUser user) {
        peers.put(peerId, user);
        connectionListeners.forEach(listener -> listener.onAdd(peerId, this));
    }

    public void removePeer(PeerId peerId) {
        peers.remove(peerId);
        connectionListeners.forEach(listener -> listener.onRemove(peerId, this));
    }

    public P2PUser getPeer(PeerId peerId) {
        return peers.get(peerId);
    }

    public Collection<LocalInUser> getAllPeers() {
        return peers.values();
    }
}
