package wang.xiaorui.local.controllers;

import io.github.palexdev.materialfx.controls.MFXButton;
import io.github.palexdev.materialfx.controls.MFXIconWrapper;
import io.github.palexdev.materialfx.controls.MFXRectangleToggleNode;
import io.github.palexdev.materialfx.controls.MFXScrollPane;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialog;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialogBuilder;
import io.github.palexdev.materialfx.dialogs.MFXStageDialog;
import io.github.palexdev.materialfx.enums.ScrimPriority;
import io.github.palexdev.materialfx.utils.ToggleButtonsUtil;
import io.github.palexdev.mfxresources.fonts.MFXFontIcon;
import io.libp2p.core.PeerId;
import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.Pos;
import javafx.scene.control.TextArea;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.ToggleGroup;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.VBox;
import javafx.stage.Modality;
import javafx.stage.Stage;
import wang.xiaorui.local.handler.MessageBuilderHandler;
import wang.xiaorui.local.server.ConnectionCache;
import wang.xiaorui.local.server.ConnectionListener;
import wang.xiaorui.local.server.LocalInMessageObserver;
import wang.xiaorui.local.server.LocalInUser;

import java.net.URL;
import java.util.*;

/**
 * @author wangxiaorui
 * @date 2025/2/10
 * @desc
 */
public class OnlineChatController implements Initializable, ConnectionListener, LocalInMessageObserver {
    @FXML
    public MFXScrollPane chatUserScrollPane;
    @FXML
    public TextArea chatTextArea;
    @FXML
    public TextArea chatInput;

    private final ToggleGroup toggleGroup;
    //    @FXML
//    public VBox chatUserListBox;
    @FXML
    public AnchorPane onlineChatPane;

    @FXML
    public VBox messageItemBox;

    private LocalInUser currentSelectUser;
    private Stage stage;

    private MFXStageDialog dialog;
    private MFXGenericDialog dialogContent;

    private OnlineChatController() {
        this.toggleGroup = new ToggleGroup();
        ToggleButtonsUtil.addAlwaysOneSelectedSupport(toggleGroup);
    }

    private static volatile OnlineChatController instance;

    public static OnlineChatController getInstance() {
        if (instance == null) {
            synchronized (OnlineChatController.class) {
                if (instance == null) {
                    instance = new OnlineChatController();
                }
            }
        }
        return instance;
    }

    public void setStage(Stage stage) {
        this.stage = stage;
    }

    public void sendMessage(ActionEvent actionEvent) {
        String text = chatInput.getText().trim();
        if (text.isEmpty()) {
            return;
        }
        //此处发送的都是群发消息
        Collection<LocalInUser> allPeers = ConnectionCache.getInstance().getAllPeers();
        for (LocalInUser user : allPeers) {
            String groupMessage = "/group" + text;
            user.getController().send(groupMessage);
        }
        chatInput.clear();
        messageItemBox.getChildren().add(MessageBuilderHandler.handleSelfMessage(text));
        System.out.println("发送一条消息");
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        Platform.runLater(() -> {
            this.dialogContent = MFXGenericDialogBuilder.build()
                    .makeScrollable(true)
                    .get();
            this.dialog = MFXGenericDialogBuilder.build(dialogContent)
                    .setShowClose(false)
                    .setShowAlwaysOnTop(false)
                    .setShowMinimize(false)
                    .toStageDialogBuilder()
                    .initOwner(stage)
                    .initModality(Modality.APPLICATION_MODAL)
                    .setDraggable(false)
                    .setTitle("提示信息")
                    .setOwnerNode(onlineChatPane)
                    .setScrimPriority(ScrimPriority.WINDOW)
                    .setScrimOwner(true)
                    .get();
            dialogContent.addActions(
                    Map.entry(new MFXButton("我已了解"), event -> dialog.close())
            );
            dialogContent.setPrefHeight(80);
            dialogContent.setMaxSize(300, 100);
        });
        //initUserList();

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

//    public void initUserList(ConnectionCache connectionCache) {
//        Platform.runLater(() -> {
//            chatUserListBox.getChildren().clear();
//        });
//        Collection<LocalInUser> allPeers = connectionCache.getAllPeers();
//        List<ToggleButton> toggleButtons = new ArrayList<>();
//        if (!allPeers.isEmpty()) {
//            for (LocalInUser user : allPeers) {
//                toggleButtons.add(createToggleAction(user));
//            }
//            Platform.runLater(() -> {
//                chatUserListBox.getChildren().setAll(toggleButtons);
//            });
//        }
//    }

//    private ToggleButton createToggleAction(LocalInUser user) {
//        List<String> hostAddress = user.getHostAddress();
//        String clientIp = hostAddress.get(0);
//        String clientName = "匿名用户" + clientIp.substring(clientIp.lastIndexOf("."));
//        ToggleButton toggle = createToggle(clientName);
//        toggle.setOnAction(actionEvent -> {
//            currentSelectUser = user;
//        });
//        return toggle;
//    }

//    private ToggleButton createToggle(String text) {
//        MFXIconWrapper wrapper = new MFXIconWrapper("fas-circle-user", 24, 32);
//        MFXRectangleToggleNode toggleNode = new MFXRectangleToggleNode(text, wrapper);
//        toggleNode.setAlignment(Pos.CENTER_LEFT);
//        toggleNode.setMaxWidth(136);
//        toggleNode.setToggleGroup(toggleGroup);
//        return toggleNode;
//    }

    @Override
    public void onAdd(PeerId peerId, ConnectionCache connectionCache) {
        //initUserList(connectionCache);
    }

    @Override
    public void onRemove(PeerId peerId, ConnectionCache connectionCache) {
        //initUserList(connectionCache);
    }

    @Override
    public void onMessage(String message) {
        Platform.runLater(()-> {
            messageItemBox.getChildren().add(MessageBuilderHandler.handleOtherMessage(message));
        });
    }

}
