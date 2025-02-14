package wang.xiaorui.local.p2p.founder.impl;

import io.libp2p.core.*;
import javafx.util.Pair;
import wang.xiaorui.local.p2p.founder.P2PPeerFounder;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;
import wang.xiaorui.local.p2p.message.P2PMessageHandlerFactory;
import wang.xiaorui.local.p2p.model.P2PUser;
import wang.xiaorui.local.p2p.protocol.P2PProtocolBinding;
import wang.xiaorui.local.p2p.protocol.P2PProtocolHandler;

import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/**
 * @author wangxiaorui
 * @date 2025/2/12
 * @desc
 */
public class P2PDefaultPeerFounder implements P2PPeerFounder {

    private static final Set<PeerId> knownNodes = new HashSet<>();

    private static final Map<PeerId, P2PUser> peers = new ConcurrentHashMap();

    private final P2PMessageHandlerFactory messageHandlerFactory;

    public P2PDefaultPeerFounder(P2PMessageHandlerFactory messageHandlerFactory) {
        this.messageHandlerFactory = messageHandlerFactory;
    }

    @Override
    public void peerFound(Host host, PeerInfo peerInfo, List<String> hostAddress) {
        if (host.getPeerId().equals(peerInfo.getPeerId()) ||
                knownNodes.contains(peerInfo.getPeerId())) {
            //不用缓存本身和已经存在的节点
            return;
        }
        knownNodes.add(peerInfo.getPeerId());
        Pair<Stream, P2PAbstractMessageHandler> connection = connection(host, peerInfo);
        if (connection == null) {
            return;
        }
        //首次连接成功之后询问对方身份
        connection.getValue().send("/local-in-who");
        peers.put(peerInfo.getPeerId(), new P2PUser(peerInfo.getPeerId().toBase58(), connection.getValue()));
    }

    private Pair<Stream, P2PAbstractMessageHandler> connection(Host host, PeerInfo peerInfo) {
        try {
            StreamPromise<? extends P2PAbstractMessageHandler> dial = new P2PProtocolBinding(new P2PProtocolHandler(messageHandlerFactory))
                    .dial(host, peerInfo.getPeerId(), peerInfo.getAddresses().get(0));
            return new Pair(dial.getStream().get(), dial.getController().get());
        } catch (Exception e) {
            return null;
        }
    }
}
