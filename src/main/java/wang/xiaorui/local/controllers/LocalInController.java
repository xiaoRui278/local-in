package wang.xiaorui.local.controllers;

import io.github.palexdev.materialfx.controls.*;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialog;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialogBuilder;
import io.github.palexdev.materialfx.dialogs.MFXStageDialog;
import io.github.palexdev.materialfx.enums.ButtonType;
import io.github.palexdev.materialfx.enums.ScrimPriority;
import io.github.palexdev.materialfx.utils.ToggleButtonsUtil;
import io.github.palexdev.materialfx.utils.others.loader.MFXLoader;
import io.github.palexdev.materialfx.utils.others.loader.MFXLoaderBean;
import io.github.palexdev.mfxresources.fonts.IconsProviders;
import io.github.palexdev.mfxresources.fonts.MFXFontIcon;
import io.libp2p.core.PeerId;
import javafx.application.Platform;
import javafx.css.PseudoClass;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.control.Label;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.ToggleGroup;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.*;
import javafx.stage.Modality;
import javafx.stage.Stage;
import wang.xiaorui.local.handler.LocalInMessageForwarder;
import wang.xiaorui.local.handler.observer.FileMessageObserver;
import wang.xiaorui.local.server.ConnectionCache;
import wang.xiaorui.local.server.LocalInUser;

import java.awt.*;
import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;

import static wang.xiaorui.local.MFXDemoResourcesLoader.loadURL;

/**
 * @author wangxiaorui
 * @date 2025/2/8
 * @desc
 */
public class LocalInController implements Initializable, FileMessageObserver {
    private final Stage stage;
    @FXML
    public MFXFontIcon githubIcon;
    @FXML
    public Label githubLink;
    private double xOffset;
    private double yOffset;
    private final ToggleGroup toggleGroup;
    @FXML
    public AnchorPane rootPane;
    @FXML
    private HBox windowHeader;
    @FXML
    public MFXFontIcon alwaysOnTopIcon;
    @FXML
    public MFXFontIcon minimizeIcon;
    @FXML
    public MFXFontIcon closeIcon;
    @FXML
    public MFXScrollPane scrollPane;
    @FXML
    public VBox navBar;
    @FXML
    public StackPane contentPane;

    public LocalInController(Stage stage) {
        this.stage = stage;
        this.toggleGroup = new ToggleGroup();
        ToggleButtonsUtil.addAlwaysOneSelectedSupport(toggleGroup);
        LocalInMessageForwarder.getInstance().addFileObserver(this);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        //窗口操作按钮
        closeIcon.addEventHandler(MouseEvent.MOUSE_CLICKED, event -> Platform.exit());
        minimizeIcon.addEventHandler(MouseEvent.MOUSE_CLICKED,
                event -> ((Stage) rootPane.getScene().getWindow()).setIconified(true));
        alwaysOnTopIcon.addEventHandler(MouseEvent.MOUSE_CLICKED, event -> {
            boolean newVal = !stage.isAlwaysOnTop();
            alwaysOnTopIcon.pseudoClassStateChanged(PseudoClass.getPseudoClass("always-on-top"), newVal);
            stage.setAlwaysOnTop(newVal);
        });

        windowHeader.setOnMousePressed(event -> {
            xOffset = stage.getX() - event.getScreenX();
            yOffset = stage.getY() - event.getScreenY();
        });
        windowHeader.setOnMouseDragged(event -> {
            stage.setX(event.getScreenX() + xOffset);
            stage.setY(event.getScreenY() + yOffset);
        });

        initializeLoader();

        githubIcon.setIconsProvider(IconsProviders.FONTAWESOME_BRANDS);
        githubIcon.setDescription("fab-github");

        githubLink.addEventHandler(MouseEvent.MOUSE_CLICKED, event -> {
            if (Desktop.isDesktopSupported()) {
                Desktop desktop = Desktop.getDesktop();
                if (desktop.isSupported(Desktop.Action.BROWSE)) {
                    try {
                        desktop.browse(new URI("https://github.com/xiaoRui278/local-in"));
                    } catch (IOException | URISyntaxException e) {
                        e.printStackTrace();
                    }
                }
            }
        });
    }

    private void initializeLoader() {
        MFXLoader loader = new MFXLoader();
        loader.addView(MFXLoaderBean.of("OnlineUser", loadURL("fxmls/OnlineUser.fxml"))
                .setBeanToNodeMapper(() -> createToggle("fas-chalkboard-user", "在线用户"))
                .setControllerFactory(c -> {
                    OnlineUserController onlineUserController = OnlineUserController.getInstance();
                    onlineUserController.setStage(stage);
                    onlineUserController.setRootPane(rootPane);
                    ConnectionCache.getInstance().addListener(onlineUserController);
                    return onlineUserController;
                })
                .setDefaultRoot(true).get());
        loader.addView(MFXLoaderBean.of("OnlineChat", loadURL("fxmls/OnlineChat.fxml"))
                .setBeanToNodeMapper(() -> createToggle("fas-comments", "在线聊天"))
                .setControllerFactory(c -> {
                    OnlineChatController onlineChatController = OnlineChatController.getInstance();
                    onlineChatController.setStage(stage);
                    //注册自己到群消息收发器
                    LocalInMessageForwarder.getInstance().addMessageObserver(onlineChatController);
                    return onlineChatController;
                })
                .get());
        loader.setOnLoadedAction(beans -> {
            List<ToggleButton> nodes = beans.stream()
                    .map(bean -> {
                        ToggleButton toggle = (ToggleButton) bean.getBeanToNodeMapper().get();
                        toggle.setOnAction(event -> contentPane.getChildren().setAll(bean.getRoot()));
                        if (bean.isDefaultView()) {
                            contentPane.getChildren().setAll(bean.getRoot());
                            toggle.setSelected(true);
                        }
                        return toggle;
                    })
                    .toList();
            navBar.getChildren().setAll(nodes);
        });
        loader.start();
    }

    private ToggleButton createToggle(String icon, String text) {
        return createToggle(icon, text, 0);
    }

    private ToggleButton createToggle(String icon, String text, double rotate) {
        MFXIconWrapper wrapper = new MFXIconWrapper(icon, 24, 32);
        MFXRectangleToggleNode toggleNode = new MFXRectangleToggleNode(text, wrapper);
        toggleNode.setAlignment(Pos.CENTER_LEFT);
        toggleNode.setMaxWidth(Double.MAX_VALUE);
        toggleNode.setToggleGroup(toggleGroup);
        toggleNode.setStyle("-fx-cursor: hand;");
        if (rotate != 0) wrapper.getIcon().setRotate(rotate);
        return toggleNode;
    }

    /**
     * 已选文件显示容器
     */
    private VBox acceptFileBox;

    @Override
    public void onAcceptFileMetaMessage(PeerId peerId, String fileName, String fileSize) {
        LocalInUser user = (LocalInUser)ConnectionCache.getInstance().getPeer(peerId);
        if(null == user){
            return;
        }
        Platform.runLater(() -> {
            // 弹出文件接收对话框
            MFXGenericDialog mfxGenericDialog = null;
            try {
                // 创建内容容器
                VBox contentContainer = new VBox(15);
                contentContainer.setPadding(new Insets(20));
                contentContainer.setAlignment(Pos.TOP_CENTER);

                acceptFileBox = new VBox();
                HBox fileItemHbox = new HBox();
                fileItemHbox.setAlignment(Pos.BASELINE_LEFT);
                fileItemHbox.setSpacing(20);
                javafx.scene.control.Label fileNameLabel =
                        new javafx.scene.control.Label(fileName + "(" + fileSize + ")");
                MFXProgressBar mfxProgressBar = new MFXProgressBar();
                mfxProgressBar.setProgress(0.4);
                mfxProgressBar.setPrefWidth(400);
                fileItemHbox.getChildren().addAll(fileNameLabel, mfxProgressBar);
                HBox.setHgrow(mfxProgressBar, Priority.ALWAYS);
                acceptFileBox.getChildren().add(fileItemHbox);
                // 组装内容
                contentContainer.getChildren().addAll(
                        acceptFileBox
                );

                MFXFontIcon warnIcon = new MFXFontIcon("fas-file-export", 18);
                mfxGenericDialog = MFXGenericDialogBuilder.build()
                        //.setContentText("文件发送对话框")
                        .setContent(contentContainer)
                        .setHeaderIcon(warnIcon)
                        .setHeaderText("收到[" + user.getHostAddress().get(0) + "]发送的文件")
                        .get();
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
            //构建dialog
            MFXStageDialog dialog = MFXGenericDialogBuilder.build(mfxGenericDialog)
                    .setShowMinimize(false)
                    .setShowAlwaysOnTop(false)
                    .setShowClose(false)
                    .toStageDialogBuilder()
                    .initOwner(stage)
                    .initModality(Modality.APPLICATION_MODAL)
                    .setDraggable(false)
                    .setOwnerNode(rootPane)
                    .setScrimPriority(ScrimPriority.NODE)
                    .setScrimOwner(true)
                    .setScrimStrength(0.5)
                    .get();

            //接收按钮
            MFXButton sendButton = new MFXButton("接收");
            sendButton.setButtonType(ButtonType.RAISED);
            sendButton.setStyle("-fx-background-color:#79BBFF; -fx-text-fill: #FFFFFF; -fx-cursor: hand; -fx-padding:" +
                    " 6 " +
                    "22;");

            //取消按钮
            MFXButton cancelButton = new MFXButton("拒绝");
            cancelButton.setButtonType(ButtonType.RAISED);
            cancelButton.setStyle("-fx-background-color:#CDD0D6; -fx-cursor: hand; -fx-padding: 6 22;");
            mfxGenericDialog.addActions(
                    Map.entry(sendButton, e -> {
                        System.out.println("=====>> 接收文件");
                    }),
                    Map.entry(cancelButton, e -> {
                        dialog.close();
                        System.out.println("=====>> 取消");
                    })
            );

            dialog.setHeight(240);
            dialog.setWidth(600);
            dialog.showDialog();
        });
    }
}
