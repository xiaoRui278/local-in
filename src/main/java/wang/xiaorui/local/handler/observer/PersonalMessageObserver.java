package wang.xiaorui.local.handler.observer;

public interface PersonalMessageObserver {

    void onMessage(String fromUser, String message);

    /**
     * 收到文件Meta消息
     */
    void onAcceptFileMetaMessage(String fromUser, String fileName, String fileSize);
}
