package wang.xiaorui.local.server;

import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;
import wang.xiaorui.local.p2p.model.P2PUser;

import java.util.List;

/**
 * @author wangxiaorui
 * @date 2025/2/13
 * @desc
 */
public class LocalInUser extends P2PUser {
    private List<String> hostAddress;

    public LocalInUser(String name, P2PAbstractMessageHandler controller, List<String> hostAddress) {
        super(name, controller);
        this.hostAddress = hostAddress;
    }

    public List<String> getHostAddress() {
        return hostAddress;
    }

    public void setHostAddress(List<String> hostAddress) {
        this.hostAddress = hostAddress;
    }
}
