package wang.xiaorui.local.controllers;

import io.github.palexdev.materialfx.controls.MFXButton;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialog;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialogBuilder;
import io.github.palexdev.materialfx.dialogs.MFXStageDialog;
import io.github.palexdev.materialfx.enums.ScrimPriority;
import io.github.palexdev.materialfx.utils.others.loader.MFXLoader;
import io.github.palexdev.mfxresources.fonts.MFXFontIcon;
import io.libp2p.core.PeerId;
import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Parent;
import javafx.scene.control.Label;
import javafx.scene.layout.*;
import javafx.stage.Modality;
import javafx.stage.Stage;
import wang.xiaorui.local.MFXDemoResourcesLoader;
import wang.xiaorui.local.handler.PersonalMessageHandler;
import wang.xiaorui.local.server.ConnectionCache;
import wang.xiaorui.local.server.ConnectionListener;
import wang.xiaorui.local.server.LocalInUser;

import java.io.IOException;
import java.net.URL;
import java.util.*;

/**
 * @author wangxiaorui
 * @date 2025/2/8
 * @desc
 */
public class OnlineUserController implements Initializable, ConnectionListener {

    @FXML
    public VBox userListVBoxNode;
    @FXML
    public AnchorPane onlineUserPane;

    private Stage stage;

    private Pane rootPane;

    private OnlineUserController() {
    }

    private static volatile OnlineUserController instance;

    public static OnlineUserController getInstance() {
        if (instance == null) {
            synchronized (OnlineUserController.class) {
                if (instance == null) {
                    instance = new OnlineUserController();
                }
            }
        }
        return instance;
    }

    public void setStage(Stage stage) {
        this.stage = stage;
    }

    public void setRootPane(Pane rootPane) {
        this.rootPane = rootPane;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        System.out.println("OnlineUserController>>>>>>>>initialize");
    }

    /**
     * 初始化用户列表
     */
    public void initUserList(ConnectionCache connectionCache) {
        Platform.runLater(() -> {
            userListVBoxNode.getChildren().clear();
        });
        Collection<LocalInUser> allPeers = connectionCache.getAllPeers();
        List<HBox> allUserHbox = new ArrayList<>();
        if (!allPeers.isEmpty()) {
            for (LocalInUser user : allPeers) {
                //正则获取IP地址
                allUserHbox.add(buildUserItem(user));
            }
            Platform.runLater(() -> {
                userListVBoxNode.getChildren().setAll(allUserHbox);
            });
        }
    }

    /**
     * 构建用户列表项
     *
     * @param user 用户IP地址
     * @return HBox
     */
    private HBox buildUserItem(LocalInUser user) {
        List<String> hostAddress = user.getHostAddress();
        String clientIp = hostAddress.get(0);
        HBox userItem = new HBox();
        userItem.getStyleClass().add("user-item-hbox");
        userItem.setPadding(new Insets(0, 20, 0, 20));
        userItem.setAlignment(Pos.CENTER);

        MFXFontIcon icon = new MFXFontIcon("fas-desktop", 40);
        icon.getStyleClass().add("user-icon");

        VBox userInfoVBox = new VBox();
        userInfoVBox.setPadding(new Insets(0, 20, 0, 20));
        userInfoVBox.setAlignment(Pos.CENTER_LEFT);

        String clientName = "匿名用户" + clientIp.substring(clientIp.lastIndexOf("."));
        Label nameLabel = new Label(clientName);
        nameLabel.setAlignment(Pos.CENTER);
        nameLabel.getStyleClass().add("user-name-label");

        Label ipLabel = new Label(clientIp);
        ipLabel.setAlignment(Pos.CENTER);
        ipLabel.setPadding(new Insets(0, 10, 0, 10));
        ipLabel.getStyleClass().add("user-name-ip");

        userInfoVBox.getChildren().setAll(nameLabel, ipLabel);

        MFXFontIcon chatIcon = new MFXFontIcon("fas-comment-dots", 30);
        chatIcon.getStyleClass().add("user-icon");
        chatIcon.setOnMouseClicked(event -> {
            System.out.println("---->点击了[" + user.getName() + "]");
            //此处直接打开一个弹框吧
            openChatWindow(user);
        });

        userItem.getChildren().addAll(icon, userInfoVBox, chatIcon);
        HBox.setHgrow(userInfoVBox, Priority.ALWAYS);
        return userItem;
    }

    public void openChatWindow(LocalInUser user) {
//    public void openChatWindow(ActionEvent event) {
        //构建Dialog内容
        FXMLLoader loader = new FXMLLoader(MFXDemoResourcesLoader.loadURL("fxmls/PersonalChat.fxml"));
        loader.setControllerFactory(c -> {
            PersonalChatController personalChatController = new PersonalChatController(user);
            PersonalMessageHandler.getInstance().addMessageObserver(user.getName(), personalChatController);
            return personalChatController;
        });
        MFXGenericDialog mfxGenericDialog = null;
        try {
            mfxGenericDialog = MFXGenericDialogBuilder.build()
                    .setContent(loader.load())
//                        .makeScrollable(true)
                    .get();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        //构建dialog
        MFXStageDialog dialog = MFXGenericDialogBuilder.build(mfxGenericDialog)
                .setShowMinimize(false)
                .setShowAlwaysOnTop(false)
                .toStageDialogBuilder()
                .initOwner(stage)
                .initModality(Modality.APPLICATION_MODAL)
                .setDraggable(false)
                .setTitle("与[" + user.getHostAddress().get(0) + "]聊天")
                .setOwnerNode(rootPane)
                .setScrimPriority(ScrimPriority.WINDOW)
                .setScrimOwner(true)
                .get();
        mfxGenericDialog.setMaxSize(800, 600);
        mfxGenericDialog.setPrefHeight(600);
        dialog.setHeight(600);
        dialog.setWidth(800);
        Platform.runLater(dialog::showDialog);
    }


    @Override
    public void onAdd(PeerId peerId, ConnectionCache connectionCache) {
        initUserList(connectionCache);
    }

    @Override
    public void onRemove(PeerId peerId, ConnectionCache connectionCache) {
        initUserList(connectionCache);
    }
}
