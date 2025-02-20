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

    /**
     * 移除链接
     * @param peerId ID
     */
    public void removePeer(PeerId peerId) {
        if(peers.containsKey(peerId)){
            peers.get(peerId).getController().getStream().close();
            peers.remove(peerId);
        }
        connectionListeners.forEach(listener -> listener.onRemove(peerId, this));
    }

    /**
     * 根据ID查询链接信息
     * @param peerId ID
     * @return 链接信息
     */
    public P2PUser getPeer(PeerId peerId) {
        return peers.get(peerId);
    }

    /**
     * 获取所有链接
     * @return 所有链接
     */
    public Collection<LocalInUser> getAllPeers() {
        return peers.values();
    }

    /**
     * 停止连接缓存，关闭连接
     */
    public void stop(){
        for (LocalInUser value : peers.values()) {
            value.getController().getStream().close();
        }
        connectionListeners.clear();
    }
}
