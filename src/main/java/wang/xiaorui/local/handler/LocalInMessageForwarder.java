package wang.xiaorui.local.handler;

import wang.xiaorui.local.handler.observer.GroupMessageObserver;
import wang.xiaorui.local.handler.observer.PersonalMessageObserver;
import wang.xiaorui.local.server.ConnectionCache;
import wang.xiaorui.local.server.LocalInUser;

import java.util.*;

/**
 * @author wangxiaorui
 * @date 2025/2/19
 * @desc
 */
public class LocalInMessageForwarder {

    /**
     * 个人消息监听
     */
    private static final Map<String, List<PersonalMessageObserver>> personalObserverMap = new HashMap<>();

    /**
     * 个人消息缓存
     */
    private static final Map<String, List<MessageCache>> personalAllMessage = new HashMap<>();


    /**
     * 群聊消息监听
     */
    private static final List<GroupMessageObserver> groupObserverMap = new ArrayList<>();

    private static final List<MessageCache> groupAllMessage = new ArrayList<>();

    private LocalInMessageForwarder() {
    }

    private static volatile LocalInMessageForwarder instance;

    public static LocalInMessageForwarder getInstance() {
        if (instance == null) {
            synchronized (LocalInMessageForwarder.class) {
                if (instance == null) {
                    instance = new LocalInMessageForwarder();
                }
            }
        }
        return instance;
    }

    /**
     * 添加个人消息监听器
     *
     * @param userName        用户name
     * @param messageObserver 监听器
     */
    public void addPersonalObserver(String userName, PersonalMessageObserver messageObserver) {
        personalObserverMap.computeIfAbsent(userName, k -> new ArrayList<>()).add(messageObserver);
    }

    /**
     * 添加群聊消息监听器
     *
     * @param messageObserver 监听器
     */
    public void addMessageObserver(GroupMessageObserver messageObserver) {
        groupObserverMap.add(messageObserver);
    }

    /**
     * 查询用户缓存消息
     *
     * @param userName 用户name
     * @return 缓存消息
     */
    public List<MessageCache> getCacheByUserName(String userName) {
        return personalAllMessage.get(userName);
    }

    /**
     * 收到消息
     *
     * @param fromUser 发送用户
     * @param message  发送消息
     */
    public void onMessage(String fromUser, String message) {
        if (message.startsWith(LocalInMessageConstants.GROUP_MESSAGE_PREFIX)) {
            //群发消息
            //去掉群发消息前缀
            message = message.substring(LocalInMessageConstants.GROUP_MESSAGE_PREFIX.length());
            //渲染消息
            onGroupMessage(fromUser, message);
            return;
        }
        //个人消息
        message = message.substring(LocalInMessageConstants.PERSONAL_MESSAGE_PREFIX.length());
        cachePersonalMessage(fromUser, message);
        if (personalObserverMap.containsKey(fromUser)) {
            List<PersonalMessageObserver> personalMessageObservers = personalObserverMap.get(fromUser);
            for (PersonalMessageObserver personalMessageObserver : personalMessageObservers) {
                personalMessageObserver.onMessage(fromUser, message);
            }
        }
    }

    /**
     * 收到群聊消息
     *
     * @param fromUser 发送用户
     * @param message  发送消息
     */
    public void onGroupMessage(String fromUser, String message) {
        //前面部分是用户
        for (GroupMessageObserver messageObserver : groupObserverMap) {
            messageObserver.onMessage(message);
        }
        cacheGroupMessage(fromUser, message);
    }

    /**
     * 发送消息
     *
     * @param user    发送用户
     * @param message 发送消息
     */
    public void sendPersonalMessage(LocalInUser user, String message) {
        String fromUser = user.getName();
        user.getController().sendMessage(message);
        cachePersonalMessage(fromUser, message);
    }

    /**
     * 发送群
     *
     * @param fromUser 发送用户
     * @param message  发送消息
     */
    public void sendGroupMessage(String fromUser, String message) {
        Collection<LocalInUser> allPeers = ConnectionCache.getInstance().getAllPeers();
        for (LocalInUser user : allPeers) {
            user.getController().sendMessageToGroup(message);
        }
        cacheGroupMessage(fromUser, message);
    }

    /**
     * 缓存个人消息
     *
     * @param cacheUser 缓存用户
     * @param message   缓存消息
     */
    private void cachePersonalMessage(String cacheUser, String message) {
        List<MessageCache> currentUserMessage = personalAllMessage.get(cacheUser);
        if (currentUserMessage == null) {
            currentUserMessage = new ArrayList<>();
            personalAllMessage.put(cacheUser, currentUserMessage);
        } else if (currentUserMessage.size() > 20) {
            // 超过20条删除最老的消息
            currentUserMessage.remove(0);
        }
        currentUserMessage.add(new MessageCache(cacheUser, message));
    }

    /**
     * 缓存群聊消息
     *
     * @param cacheUser 缓存用户
     * @param message   缓存消息
     */
    private void cacheGroupMessage(String cacheUser, String message) {
        if (groupAllMessage.size() > 50) {
            groupAllMessage.remove(0);
        }
        groupAllMessage.add(new MessageCache(cacheUser, message));
    }
}
