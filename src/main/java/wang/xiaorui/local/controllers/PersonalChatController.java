package wang.xiaorui.local.controllers;

import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.TextArea;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.VBox;
import wang.xiaorui.local.handler.LocalInMessageForwarder;
import wang.xiaorui.local.handler.MessageBuilderHandler;
import wang.xiaorui.local.handler.MessageCache;
import wang.xiaorui.local.handler.observer.PersonalMessageObserver;
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
        //发送个人消息
        LocalInMessageForwarder.getInstance().sendPersonalMessage(user, text);
        chatInput.clear();
        messageItemBox.getChildren().add(MessageBuilderHandler.handleSelfMessage(text));
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        List<MessageCache> cacheByUserName =
                LocalInMessageForwarder.getInstance().getCacheByUserName(user.getName());
        if (cacheByUserName != null) {
            for (MessageCache messageCache : cacheByUserName) {
                if (user.getName().equals(messageCache.getUserName())) {
                    messageItemBox.getChildren().add(MessageBuilderHandler.handleOtherMessage(messageCache.getMessage()));
                } else {
                    messageItemBox.getChildren().add(MessageBuilderHandler.handleSelfMessage(messageCache.getMessage()));
                }
            }
        }

        //回车发送
        chatInput.setOnKeyPressed(new EventHandler<KeyEvent>() {
            @Override
            public void handle(KeyEvent event) {
                if (event.getCode() == KeyCode.ENTER) {
                    sendMessage(null);
                    event.consume();
                }
            }
        });
    }

    @Override
    public void onMessage(String fromUser, String message) {
        Platform.runLater(() -> {
            messageItemBox.getChildren().add(MessageBuilderHandler.handleOtherMessage(message));
        });
    }
}
