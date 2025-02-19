package wang.xiaorui.local.handler.observer;

/**
 * 消息观察着
 */
public interface GroupMessageObserver {
    /**
     * 收到消息
     * @param message 消息正文
     */
    void onMessage(String message);
}
