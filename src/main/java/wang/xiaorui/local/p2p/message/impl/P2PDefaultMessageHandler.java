package wang.xiaorui.local.p2p.message.impl;

import io.libp2p.core.Stream;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;

import java.nio.charset.StandardCharsets;

/**
 * @author wangxiaorui
 * @date 2025/2/12
 * @desc
 */
public class P2PDefaultMessageHandler extends P2PAbstractMessageHandler {

    public P2PDefaultMessageHandler(Stream stream) {
        super(stream);
    }

    @Override
    public void onActivated(Stream stream) {
    }

    @Override
    public void onClosed(Stream stream) {
    }

    @Override
    public void onException(Throwable cause) {
    }

    @Override
    public void onMessage(Stream stream, ByteBuf msg) {
        String string = msg.toString(StandardCharsets.UTF_8);
    }

    @Override
    public void send(String message) {
        byte[] bytes = message.getBytes(StandardCharsets.UTF_8);
        ByteBuf messageBuf = Unpooled.wrappedBuffer(bytes);// Unpooled.copiedBuffer(message, StandardCharsets.UTF_8);
        stream.writeAndFlush(messageBuf);
    }
}
