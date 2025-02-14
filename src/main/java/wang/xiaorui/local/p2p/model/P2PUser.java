package wang.xiaorui.local.p2p.model;

import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;

/**
 * @author wangxiaorui
 * @date 2025/2/12
 * @desc
 */
public class P2PUser {

    private String name;

    private P2PAbstractMessageHandler controller;

    public P2PUser(String name, P2PAbstractMessageHandler controller) {
        this.name = name;
        this.controller = controller;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public P2PAbstractMessageHandler getController() {
        return controller;
    }

    public void setController(P2PAbstractMessageHandler controller) {
        this.controller = controller;
    }
}
