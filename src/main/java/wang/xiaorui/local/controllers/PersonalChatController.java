package wang.xiaorui.local.controllers;

import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.TextArea;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.VBox;
import wang.xiaorui.local.handler.MessageBuilderHandler;
import wang.xiaorui.local.handler.MessageCache;
import wang.xiaorui.local.handler.PersonalMessageHandler;
import wang.xiaorui.local.handler.PersonalMessageObserver;
import wang.xiaorui.local.server.LocalInUser;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

/**
 * @author wangxiaorui
 * @date 2025/2/18
 * @desc
 */
public class PersonalChatController implements Initializable, PersonalMessageObserver {
    @FXML
    public AnchorPane personalChatPane;
    @FXML
    public VBox messageItemBox;
    @FXML
    public TextArea chatInput;

    private final LocalInUser user;

    public PersonalChatController(LocalInUser user) {
        this.user = user;
    }

    public void sendMessage(ActionEvent actionEvent) {
        String text = chatInput.getText().trim();
        if (text.isEmpty()) {
            return;
        }
        //此处发送的都是群发消息
        String personalMessage = "/personal/" + user.getName() + "/" + text;
        user.getController().send(personalMessage);
        chatInput.clear();
        messageItemBox.getChildren().add(MessageBuilderHandler.handleSelfMessage(text));
        PersonalMessageHandler.getInstance().sendMessage(user.getName(), text);
        System.out.println("发送一条消息");
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        System.out.println("--PersonalChatController.initialize--->");
        List<MessageCache> cacheByUserName =
                PersonalMessageHandler.getInstance().getCacheByUserName(user.getName());
        if (cacheByUserName != null) {
            for (MessageCache messageCache : cacheByUserName) {
                if (user.getName().equals(messageCache.getUserName())) {
                    messageItemBox.getChildren().add(MessageBuilderHandler.handleOtherMessage(messageCache.getMessage()));
                } else {
                    messageItemBox.getChildren().add(MessageBuilderHandler.handleSelfMessage(messageCache.getMessage()));
                }
            }
        }
    }

    @Override
    public void onMessage(String message) {
        Platform.runLater(() -> {
            messageItemBox.getChildren().add(MessageBuilderHandler.handleOtherMessage(message));
        });
    }
}
