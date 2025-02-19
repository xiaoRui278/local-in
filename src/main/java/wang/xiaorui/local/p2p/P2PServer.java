package wang.xiaorui.local.p2p;

import io.libp2p.core.Host;
import io.libp2p.core.dsl.HostBuilder;
import wang.xiaorui.local.p2p.discovery.P2PDiscoverer;
import wang.xiaorui.local.p2p.founder.P2PPeerFounder;
import wang.xiaorui.local.p2p.founder.impl.P2PDefaultPeerFounder;
import wang.xiaorui.local.p2p.message.P2PMessageHandlerFactory;
import wang.xiaorui.local.p2p.message.impl.P2PDefaultMessageHandlerFactory;
import wang.xiaorui.local.p2p.protocol.P2PProtocolBinding;
import wang.xiaorui.local.p2p.protocol.P2PProtocolHandler;
import wang.xiaorui.local.utils.NetworkUtil;

import java.io.IOException;
import java.net.InetAddress;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ExecutionException;

/**
 * @author wangxiaorui
 * @date 2025/2/11
 * @desc P2P服务
 */
public class P2PServer {

    private Host hostInstance;

    private P2PDiscoverer p2PDiscoverer;

    private final InetAddress localInetAddress;

    private final P2PMessageHandlerFactory messageHandlerFactoryInstance;

    private List<P2PPeerFounder> allPeerFounders = new ArrayList<>();

    public P2PServer() throws Exception {
        localInetAddress = NetworkUtil.getLocalInetAddress();
        String hostAddress = localInetAddress.getHostAddress();
        this.messageHandlerFactoryInstance = getMessageHandlerFactory();
    }

    /**
     * 启动P2P服务
     *
     * @throws IOException
     */
    public void start() throws IOException {
        hostInstance = new HostBuilder()
                .protocol(new P2PProtocolBinding(new P2PProtocolHandler(messageHandlerFactoryInstance)))
                .listen("/ip4/" + localInetAddress.getHostAddress() + "/tcp/0")
                .build();
        hostInstance.start().join();
        System.out.println("P2P Node started and listening on " + hostInstance.listenAddresses());
        //服务发现
        p2PDiscoverer = new P2PDiscoverer(hostInstance);
        //注册节点发现器
        addPeerFounder(allPeerFounders, messageHandlerFactoryInstance);
        for (P2PPeerFounder peerFounder : allPeerFounders) {
            p2PDiscoverer.addPeerFounder(peerFounder);
        }
        p2PDiscoverer.start();
    }

    /**
     * 停止P2P服务
     *
     * @throws ExecutionException
     * @throws InterruptedException
     */
    public void stop() throws ExecutionException, InterruptedException {
        p2PDiscoverer.stop();
        hostInstance.stop().get();
        System.out.println("P2P Node stopped");
    }

    public void addPeerFounder(List<P2PPeerFounder> peerFounders, P2PMessageHandlerFactory messageHandlerFactory) {
        peerFounders.add(new P2PDefaultPeerFounder(messageHandlerFactory));
    }

    public P2PMessageHandlerFactory getMessageHandlerFactory() {
        return new P2PDefaultMessageHandlerFactory();
    }
}
