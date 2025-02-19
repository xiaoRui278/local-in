package wang.xiaorui.local.handler;

/**
 * @author wangxiaorui
 * @date 2025/2/19
 * @desc
 */
public class MessageCache {
    private String userName;

    private String message;

    public MessageCache(String userName, String message) {
        this.userName = userName;
        this.message = message;
    }

    public String getUserName() {
        return userName;
    }

    public void setUserName(String userName) {
        this.userName = userName;
    }

    public String getMessage() {
        return message;
    }

    public void setMessage(String message) {
        this.message = message;
    }
}
