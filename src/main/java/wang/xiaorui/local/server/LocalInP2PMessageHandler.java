package wang.xiaorui.local.server;

import io.libp2p.core.Stream;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import wang.xiaorui.local.handler.LocalInMessageConstants;
import wang.xiaorui.local.handler.LocalInMessageForwarder;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;

import java.nio.charset.StandardCharsets;

/**
 * @author wangxiaorui
 * @date 2025/2/13
 * @desc
 */
public class LocalInP2PMessageHandler extends P2PAbstractMessageHandler {
    private final ConnectionCache connectionCache;

    public LocalInP2PMessageHandler(Stream stream) {
        super(stream);
        this.connectionCache = ConnectionCache.getInstance();
    }

    @Override
    public void onActivated(Stream stream) {
    }

    @Override
    public void onClosed(Stream stream) {
        connectionCache.removePeer(stream.remotePeerId());
    }

    @Override
    public void onException(Throwable cause) {
    }

    @Override
    public void onMessage(Stream stream, ByteBuf msg) {
        String fromUser = stream.remotePeerId().toBase58();
        String message = msg.toString(StandardCharsets.UTF_8);
        if (null == message || message.isEmpty()) {
            return;
        }
        LocalInMessageForwarder.getInstance().onMessage(fromUser, message);
    }

    @Override
    public void sendMessage(String message) {
        String newMessage = LocalInMessageConstants.PERSONAL_MESSAGE_PREFIX + message;
        send(newMessage);
    }

    @Override
    public void sendMessageToGroup(String message) {
        String newMessage = LocalInMessageConstants.GROUP_MESSAGE_PREFIX + message;
        send(newMessage);
    }

    private void send(String message) {
        byte[] bytes = message.getBytes(StandardCharsets.UTF_8);
        ByteBuf messageBuf = Unpooled.wrappedBuffer(bytes);// Unpooled.copiedBuffer(message, StandardCharsets.UTF_8);
        stream.writeAndFlush(messageBuf);
    }
}
