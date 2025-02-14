package wang.xiaorui.local.controllers;

import io.github.palexdev.mfxresources.fonts.MFXFontIcon;
import io.libp2p.core.PeerId;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.control.Label;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import wang.xiaorui.local.server.ConnectionCache;
import wang.xiaorui.local.server.ConnectionListener;
import wang.xiaorui.local.server.LocalInUser;

import java.net.URL;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.ResourceBundle;

/**
 * @author wangxiaorui
 * @date 2025/2/8
 * @desc
 */
public class OnlineUserController implements Initializable, ConnectionListener {

    @FXML
    public VBox userListVBoxNode;

    private Stage stage;

    private OnlineUserController(){
    }

    private static volatile OnlineUserController instance;

    /**
     * 获取 OnlineUserController 的单例实例
     *
     * @return OnlineUserController 的单例实例
     */
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
                List<String> hostAddress = user.getHostAddress();
                //正则获取IP地址
                allUserHbox.add(buildUserItem(hostAddress.get(0)));
            }
            Platform.runLater(()-> {
                userListVBoxNode.getChildren().setAll(allUserHbox);
            });
        }
    }

    /**
     * 构建用户列表项
     *
     * @param clientIp 用户IP地址
     * @return HBox
     */
    private HBox buildUserItem(String clientIp) {
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

        userItem.getChildren().addAll(icon, userInfoVBox, chatIcon);
        HBox.setHgrow(userInfoVBox, Priority.ALWAYS);
        return userItem;
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
