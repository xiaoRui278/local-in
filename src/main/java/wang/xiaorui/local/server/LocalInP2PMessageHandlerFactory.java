package wang.xiaorui.local.server;

import io.libp2p.core.Stream;
import wang.xiaorui.local.controllers.OnlineChatController;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;
import wang.xiaorui.local.p2p.message.P2PMessageHandlerFactory;

/**
 * @author wangxiaorui
 * @date 2025/2/13
 * @desc
 */
public class LocalInP2PMessageHandlerFactory implements P2PMessageHandlerFactory {

    private static volatile LocalInP2PMessageHandlerFactory instance;

    private LocalInP2PMessageHandlerFactory() {
    }

    public static LocalInP2PMessageHandlerFactory getInstance() {
        if (instance == null) {
            synchronized (LocalInP2PMessageHandlerFactory.class) {
                if (instance == null) {
                    instance = new LocalInP2PMessageHandlerFactory();
                }
            }
        }
        return instance;
    }

    @Override
    public P2PAbstractMessageHandler create(Stream stream) {
        LocalInP2PMessageHandler localInP2PMessageHandler = new LocalInP2PMessageHandler(stream);
        localInP2PMessageHandler.addMessageObserver(OnlineChatController.getInstance());
        return localInP2PMessageHandler;
    }
}
