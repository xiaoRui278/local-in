package wang.xiaorui.local.controllers;

import io.github.palexdev.materialfx.dialogs.MFXGenericDialog;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialogBuilder;
import io.github.palexdev.materialfx.dialogs.MFXStageDialog;
import io.github.palexdev.materialfx.enums.ScrimPriority;
import io.github.palexdev.materialfx.utils.AnimationUtils;
import io.github.palexdev.mfxresources.fonts.MFXFontIcon;
import io.libp2p.core.PeerId;
import javafx.animation.KeyFrame;
import javafx.animation.KeyValue;
import javafx.animation.PauseTransition;
import javafx.animation.Timeline;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.FXMLLoader;
import javafx.fxml.Initializable;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.layout.*;
import javafx.stage.Modality;
import javafx.stage.Stage;
import javafx.util.Duration;
import wang.xiaorui.local.MFXDemoResourcesLoader;
import wang.xiaorui.local.constants.Constants;
import wang.xiaorui.local.handler.LocalInMessageForwarder;
import wang.xiaorui.local.handler.observer.PersonalMessageObserver;
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
public class OnlineUserController implements Initializable, ConnectionListener, PersonalMessageObserver {

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
                LocalInMessageForwarder.getInstance().addPersonalObserver(user.getName(), this);
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
        userItem.setId(Constants.USER_ITEM_HBOX_PREFIX + user.getName());
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
        chatIcon.setId(Constants.USER_CHAT_ICON_PREFIX + user.getName());
        chatIcon.getStyleClass().add("user-icon");
        chatIcon.setOnMouseClicked(event -> {
            //此处直接打开一个弹框吧
            openChatWindow(user);
        });

        userItem.getChildren().addAll(icon, userInfoVBox, chatIcon);
        HBox.setHgrow(userInfoVBox, Priority.ALWAYS);
        return userItem;
    }

    public void openChatWindow(LocalInUser user) {
        //构建Dialog内容
        FXMLLoader loader = new FXMLLoader(MFXDemoResourcesLoader.loadURL("fxmls/PersonalChat.fxml"));
        loader.setControllerFactory(c -> {
            PersonalChatController personalChatController = new PersonalChatController(user);
            LocalInMessageForwarder.getInstance().addPersonalObserver(user.getName(), personalChatController);
            return personalChatController;
        });
        MFXGenericDialog mfxGenericDialog = null;
        try {
            mfxGenericDialog = MFXGenericDialogBuilder.build()
                    .setContent(loader.load())
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

    private static final Set<String> flashing = new HashSet<>();

    @Override
    public void onMessage(String fromUser, String message) {
        //收到消息刷新页面样式，让用户知道收到了消息
        userListVBoxNode.getChildren().forEach(hBox -> {
            if (hBox.getId().equals(Constants.USER_ITEM_HBOX_PREFIX + fromUser) && !flashing.contains(fromUser)) {
                flashing.add(fromUser);
                Timeline timelineAnimation = getTimelineAnimation(hBox);
                Platform.runLater(() -> {
                    // 启动动画
                    timelineAnimation.play();
                    PauseTransition pause = new PauseTransition(Duration.seconds(5));
                    pause.setOnFinished(event -> {
                        timelineAnimation.stop();
                        flashing.remove(fromUser);
                    });
                    pause.play();
                });
            }
        });

    }

    /**
     * 获取动画
     *
     * @return
     */
    private Timeline getTimelineAnimation(Node hBox) {
        AnimationUtils.TimelineBuilder timelineBuilder = AnimationUtils.TimelineBuilder.build()
                .add(
                        AnimationUtils.KeyFrames.of(Duration.ZERO, hBox.styleProperty(), "-fx-background-color: -mfx-blue-secondary;"),
                        AnimationUtils.KeyFrames.of(Duration.millis(500), hBox.styleProperty(), "-fx-background-color: -mfx-blue-tertiary;"),
                        AnimationUtils.KeyFrames.of(Duration.millis(1000), hBox.styleProperty(), "-fx-background-color: -mfx-blue-secondary;"),
                        AnimationUtils.KeyFrames.of(Duration.millis(1500), hBox.styleProperty(), "-fx-background-color: -mfx-blue-tertiary;")
                ).setCycleCount(Timeline.INDEFINITE);
        return timelineBuilder.getAnimation();
    }
}
