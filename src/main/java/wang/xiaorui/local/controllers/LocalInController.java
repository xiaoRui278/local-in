package wang.xiaorui.local.controllers;

import io.github.palexdev.materialfx.controls.MFXIconWrapper;
import io.github.palexdev.materialfx.controls.MFXRectangleToggleNode;
import io.github.palexdev.materialfx.controls.MFXScrollPane;
import io.github.palexdev.materialfx.utils.ToggleButtonsUtil;
import io.github.palexdev.materialfx.utils.others.loader.MFXLoader;
import io.github.palexdev.materialfx.utils.others.loader.MFXLoaderBean;
import io.github.palexdev.mfxresources.fonts.MFXFontIcon;
import javafx.application.Platform;
import javafx.css.PseudoClass;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.Pos;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.ToggleGroup;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.StackPane;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import wang.xiaorui.local.handler.LocalInMessageForwarder;
import wang.xiaorui.local.server.ConnectionCache;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

import static wang.xiaorui.local.MFXDemoResourcesLoader.loadURL;

/**
 * @author wangxiaorui
 * @date 2025/2/8
 * @desc
 */
public class LocalInController implements Initializable {
    private final Stage stage;
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
}
