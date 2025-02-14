package wang.xiaorui.local.p2p.protocol;

import io.libp2p.core.P2PChannelHandler;
import io.libp2p.core.multistream.StrictProtocolBinding;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;

/**
 * @author wangxiaorui
 * @date 2025/2/11
 * @desc
 */
public class P2PProtocolBinding extends StrictProtocolBinding<P2PAbstractMessageHandler> {

    private static final String PROTOCOL_ID = "/local-in/chat/0.1.0";

    public P2PProtocolBinding(P2PChannelHandler<P2PAbstractMessageHandler> protocol) {
        super(PROTOCOL_ID, protocol);
    }


}
