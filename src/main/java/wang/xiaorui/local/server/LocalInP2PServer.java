package wang.xiaorui.local.server;

import wang.xiaorui.local.founder.LocalInPeerFounder;
import wang.xiaorui.local.p2p.P2PServer;
import wang.xiaorui.local.p2p.founder.P2PPeerFounder;
import wang.xiaorui.local.p2p.message.P2PMessageHandlerFactory;

import java.util.List;

/**
 * @author wangxiaorui
 * @date 2025/2/13
 * @desc
 */
public class LocalInP2PServer extends P2PServer {
    public LocalInP2PServer() throws Exception {
        super();
    }

    @Override
    public void addPeerFounder(List<P2PPeerFounder> peerFounders, P2PMessageHandlerFactory messageHandlerFactory) {
        peerFounders.add(new LocalInPeerFounder(messageHandlerFactory));
    }

    @Override
    public P2PMessageHandlerFactory getMessageHandlerFactory() {
        return LocalInP2PMessageHandlerFactory.getInstance();
    }
}
