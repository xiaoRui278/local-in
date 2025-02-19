package wang.xiaorui.local.handler;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * @author wangxiaorui
 * @date 2025/2/19
 * @desc
 */
public class PersonalMessageHandler {

    private static final Map<String, PersonalMessageObserver> observerMap = new HashMap<>();

    private static final Map<String, List<MessageCache>> allMessage = new HashMap<>();

    private PersonalMessageHandler() {
    }

    private static volatile PersonalMessageHandler instance;

    public static PersonalMessageHandler getInstance() {
        if (instance == null) {
            synchronized (PersonalMessageHandler.class) {
                if (instance == null) {
                    instance = new PersonalMessageHandler();
                }
            }
        }
        return instance;
    }

    public void addMessageObserver(String userName, PersonalMessageObserver messageObserver) {
        observerMap.put(userName, messageObserver);
    }

    public List<MessageCache> getCacheByUserName(String userName) {
        return allMessage.get(userName);
    }

    public void onMessage(String fromUser, String message) {
        //前面部分是用户
        if (observerMap.containsKey(fromUser)) {
            observerMap.get(fromUser).onMessage(message);
            cacheMessage(fromUser, message);
        }
    }

    public void sendMessage(String toUser, String message) {
        cacheMessage(toUser, message);
    }

    private void cacheMessage(String cacheUser, String message) {
        List<MessageCache> currentUserMessage = allMessage.get(cacheUser);
        if (currentUserMessage == null) {
            currentUserMessage = new ArrayList<>();
            allMessage.put(cacheUser, currentUserMessage);
        } else if (currentUserMessage.size() > 20) {
            // 超过20条删除最老的消息
            currentUserMessage.remove(0);
        }
        currentUserMessage.add(new MessageCache(cacheUser, message));
    }
}
