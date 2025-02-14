package wang.xiaorui.local.handler;

import io.github.palexdev.mfxresources.fonts.MFXFontIcon;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.control.Label;
import javafx.scene.layout.HBox;

/**
 * @author wangxiaorui
 * @date 2025/2/14
 * @desc
 */
public class MessageBuilderHandler {

    public static HBox handleOtherMessage(String message) {
        // 创建HBox并设置样式类和填充
        HBox messageItem = new HBox();
        messageItem.getStyleClass().add("message-item");
        messageItem.setPadding(new Insets(5, 5, 5, 5));

        // 创建头像
        MFXFontIcon avatar = new MFXFontIcon();
        avatar.setDescription("fas-circle-user");
        avatar.setSize(24.0);
        avatar.getStyleClass().add("message-sender");

        // 创建消息正文
        Label messageContent = new Label(message);
        messageContent.getStyleClass().add("message-item-content");
        HBox.setMargin(messageContent, new Insets(8, 0, 0, 10));
        messageContent.setPadding(new Insets(5, 10, 5, 10));

        // 将头像和消息正文添加到HBox
        messageItem.getChildren().addAll(avatar, messageContent);
        return messageItem;
    }

    public static HBox handleSelfMessage(String message) {
        // 创建HBox并设置样式类、填充和对齐方式
        HBox messageItem = new HBox();
        messageItem.getStyleClass().add("message-item");
        messageItem.setPadding(new Insets(5, 5, 5, 5));
        messageItem.setAlignment(Pos.TOP_RIGHT);

        // 创建消息正文
        Label messageContent = new Label(message);
        messageContent.getStyleClass().addAll("message-item-content", "me");
        HBox.setMargin(messageContent, new Insets(8, 10, 0, 10));
        messageContent.setPadding(new Insets(5, 10, 5, 10));

        // 创建头像
        MFXFontIcon avatar = new MFXFontIcon();
        avatar.setDescription("fas-circle-user");
        avatar.setSize(24.0);
        avatar.getStyleClass().add("message-sender");

        // 将消息正文和头像添加到HBox
        messageItem.getChildren().addAll(messageContent, avatar);

        return messageItem;
    }
}
