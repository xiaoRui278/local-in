package wang.xiaorui.local.p2p.message.impl;

import io.libp2p.core.Stream;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;
import wang.xiaorui.local.p2p.message.P2PMessageHandlerFactory;

/**
 * @author wangxiaorui
 * @date 2025/2/13
 * @desc
 */
public class P2PDefaultMessageHandlerFactory implements P2PMessageHandlerFactory {
    @Override
    public P2PAbstractMessageHandler create(Stream stream) {
        return new P2PDefaultMessageHandler(stream);
    }
}
