package wang.xiaorui.local.p2p.message;

import io.libp2p.core.Stream;

/**
 * 创建MessageHandler
 */
public interface P2PMessageHandlerFactory {
    P2PAbstractMessageHandler create(Stream stream);
}
