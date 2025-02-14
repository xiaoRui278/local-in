package wang.xiaorui.local.p2p.message;

import io.libp2p.core.Stream;
import io.libp2p.protocol.ProtocolMessageHandler;
import io.netty.buffer.ByteBuf;

/**
 * 消息处理接口
 */
public abstract class P2PAbstractMessageHandler implements ProtocolMessageHandler<ByteBuf> {

    protected final Stream stream;

    public P2PAbstractMessageHandler(Stream stream) {
        this.stream = stream;
    }

    public Stream getStream() {
        return stream;
    }

    /**
     * 发送消息
     *
     * @param message 消息
     */
    public abstract void send(String message);
}
