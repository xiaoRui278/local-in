package wang.xiaorui.local.p2p.protocol;

import io.libp2p.core.Stream;
import io.libp2p.protocol.ProtocolHandler;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;
import wang.xiaorui.local.p2p.message.P2PMessageHandlerFactory;

import java.util.concurrent.CompletableFuture;

/**
 * @author wangxiaorui
 * @date 2025/2/11
 * @desc
 */
public class P2PProtocolHandler extends ProtocolHandler<P2PAbstractMessageHandler> {

    private final P2PMessageHandlerFactory messageHandlerFactory;

    public P2PProtocolHandler(P2PMessageHandlerFactory messageHandlerFactory) {
        super(Long.MAX_VALUE, Long.MAX_VALUE);
        this.messageHandlerFactory = messageHandlerFactory;
    }

    @Override
    protected CompletableFuture<P2PAbstractMessageHandler> onStartInitiator(Stream stream) {
        System.out.println("----P2PProtocolHandler-onStartInitiator---->");
        P2PAbstractMessageHandler sender = messageHandlerFactory.create(stream);
        stream.pushHandler(sender);
        return CompletableFuture.completedFuture(sender);
    }

    @Override
    protected CompletableFuture<P2PAbstractMessageHandler> onStartResponder(Stream stream) {
        System.out.println("----P2PProtocolHandler-onStartResponder---->");
        P2PAbstractMessageHandler receiver =  messageHandlerFactory.create(stream);
        stream.pushHandler(receiver);
        return CompletableFuture.completedFuture(receiver);
    }
}
