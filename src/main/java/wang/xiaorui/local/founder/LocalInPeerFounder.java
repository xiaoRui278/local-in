package wang.xiaorui.local.founder;

import io.libp2p.core.Host;
import io.libp2p.core.PeerInfo;
import io.libp2p.core.Stream;
import io.libp2p.core.StreamPromise;
import javafx.util.Pair;
import wang.xiaorui.local.p2p.founder.P2PPeerFounder;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;
import wang.xiaorui.local.p2p.message.P2PMessageHandlerFactory;
import wang.xiaorui.local.p2p.protocol.P2PProtocolBinding;
import wang.xiaorui.local.p2p.protocol.P2PProtocolHandler;
import wang.xiaorui.local.server.ConnectionCache;
import wang.xiaorui.local.server.LocalInUser;

import java.util.List;

/**
 * @author wangxiaorui
 * @date 2025/2/13
 * @desc
 */
public class LocalInPeerFounder implements P2PPeerFounder {

    private final P2PMessageHandlerFactory messageHandlerFactory;
    private final ConnectionCache connectionCache;

    public LocalInPeerFounder(P2PMessageHandlerFactory messageHandlerFactory) {
        this.messageHandlerFactory = messageHandlerFactory;
        this.connectionCache = ConnectionCache.getInstance();
    }

    @Override
    public void peerFound(Host host, PeerInfo peerInfo, List<String> hostAddress) {
        if (host.getPeerId().equals(peerInfo.getPeerId()) ||
                connectionCache.isKnownNode(peerInfo.getPeerId())) {
            //不用缓存本身和已经存在的节点
            return;
        }
        connectionCache.addKnownNode(peerInfo.getPeerId());
        Pair<Stream, P2PAbstractMessageHandler> connection = connection(host, peerInfo);
        if (connection == null) {
            return;
        }
        //首次连接成功之后询问对方身份
        connection.getValue().send("/local-in-who");
        connectionCache.addPeer(peerInfo.getPeerId(), new LocalInUser(peerInfo.getPeerId().toBase58(),
                connection.getValue(), hostAddress));
    }

    private Pair<Stream, P2PAbstractMessageHandler> connection(Host host, PeerInfo peerInfo) {
        try {
            StreamPromise<? extends P2PAbstractMessageHandler> dial =
                    new P2PProtocolBinding(new P2PProtocolHandler(messageHandlerFactory))
                    .dial(host, peerInfo.getPeerId(), peerInfo.getAddresses().get(0));
            return new Pair(dial.getStream().get(), dial.getController().get());
        } catch (Exception e) {
            return null;
        }
    }
}
