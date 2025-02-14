package wang.xiaorui.local.server;

/**
 * 消息观察着
 */
public interface LocalInMessageObserver {
    /**
     * 收到消息
     * @param message 消息正文
     */
    void onMessage(String message);
}
