package wang.xiaorui.local.controllers;

import io.github.palexdev.materialfx.controls.MFXButton;
import io.github.palexdev.materialfx.controls.MFXProgressBar;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialog;
import io.github.palexdev.materialfx.dialogs.MFXGenericDialogBuilder;
import io.github.palexdev.materialfx.dialogs.MFXStageDialog;
import io.github.palexdev.materialfx.enums.ButtonType;
import io.github.palexdev.materialfx.enums.ScrimPriority;
import io.github.palexdev.mfxresources.fonts.MFXFontIcon;
import javafx.application.Platform;
import javafx.event.ActionEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.control.TextArea;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import javafx.stage.FileChooser;
import javafx.stage.Modality;
import javafx.stage.Stage;
import wang.xiaorui.local.handler.LocalInMessageForwarder;
import wang.xiaorui.local.handler.MessageBuilderHandler;
import wang.xiaorui.local.handler.MessageCache;
import wang.xiaorui.local.handler.observer.PersonalMessageObserver;
import wang.xiaorui.local.server.LocalInUser;

import java.awt.*;
import java.io.File;
import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
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

    private Stage stage;

    private final LocalInUser user;

    public PersonalChatController(LocalInUser user) {
        this.user = user;
    }

    public void setStage(Stage stage) {
        this.stage = stage;
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



    private List<File> files = new ArrayList<>();

    /**
     * 已选文件显示容器
     */
    private VBox selectedFileBox;

    /**
     * 打开发送文件对话框发送文件
     *
     * @param event
     */
    public void openSendFileDialog(ActionEvent event) {
        MFXGenericDialog mfxGenericDialog = null;
        try {
            // 创建内容容器
            VBox contentContainer = new VBox(15);
            contentContainer.setPadding(new Insets(20));
            contentContainer.setAlignment(Pos.TOP_CENTER);

            // 创建内容按钮
            MFXButton selectFileBtn = new MFXButton("选择文件");
            MFXFontIcon mfxFontIcon = new MFXFontIcon("fas-file-circle-plus", 16);
            mfxFontIcon.setStyle("-mfx-color: white;");
            selectFileBtn.setGraphic(mfxFontIcon);
            selectFileBtn.setPrefSize(120, 40);
            selectFileBtn.setButtonType(ButtonType.RAISED);
            selectFileBtn.setStyle("-fx-background-color: #409EFF; -fx-text-fill: white; -fx-cursor: hand;");
            selectedFileBox = new VBox();
            // 按钮点击事件
            selectFileBtn.setOnAction(e -> {
                files.clear();
                FileChooser fileChooser = new FileChooser();
                File file = fileChooser.showOpenDialog(stage);
                if (file != null) {
                    files.add(file);
                    renderFileList();
                }
            });

            // 组装内容
            contentContainer.getChildren().addAll(
                    selectFileBtn,
                    selectedFileBox
            );

            MFXFontIcon warnIcon = new MFXFontIcon("fas-file-export", 18);
            mfxGenericDialog = MFXGenericDialogBuilder.build()
                    //.setContentText("文件发送对话框")
                    .setContent(contentContainer)
                    .setHeaderIcon(warnIcon)
                    .setHeaderText("发送文件到[" + user.getHostAddress().get(0) + "]")
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
                .setOwnerNode(personalChatPane)
                .setScrimPriority(ScrimPriority.NODE)
                .setScrimOwner(true)
                .setScrimStrength(0.5)
                .get();

        //发送按钮
        MFXButton sendButton = new MFXButton("发送");
        sendButton.setButtonType(ButtonType.RAISED);
        sendButton.setStyle("-fx-background-color:#79BBFF; -fx-text-fill: #FFFFFF; -fx-cursor: hand; -fx-padding: 6 " +
                "22;");

        //取消按钮
        MFXButton cancelButton = new MFXButton("取消");
        cancelButton.setButtonType(ButtonType.RAISED);
        cancelButton.setStyle("-fx-background-color:#CDD0D6; -fx-cursor: hand; -fx-padding: 6 22;");
        mfxGenericDialog.addActions(
                Map.entry(sendButton, e -> {
                    if (files.isEmpty()) {
                        return;
                    }
                    //发送文件
                    sendFiles();
                }),
                Map.entry(cancelButton, e -> dialog.close())
        );

        dialog.setHeight(240);
        dialog.setWidth(600);
        Platform.runLater(dialog::showDialog);
    }

    private void sendFiles() {
        if (files.isEmpty()) {
            return;
        }
        //目前只有一个
        File file = files.get(0);
        //1.首先发送一个消息，提示对方，有文件要发送
        //文件大小展示
        String fileSize = formatFileSize(file.length());
        LocalInMessageForwarder.getInstance().sendFileMetaMessage(user, file.getName(), fileSize);
        //2.对方点击接受，才发送文件
    }

    /**
     * 渲染文件列表界面
     */
    private void renderFileList() {
        selectedFileBox.getChildren().clear();
        if (files.isEmpty()) {
            return;
        }
        List<HBox> allSelectedFile = new ArrayList<>();
        for (File file : files) {
            String name = file.getName();
            //已选择文件
            HBox fileItemHbox = new HBox();
            fileItemHbox.setAlignment(Pos.BASELINE_LEFT);
            fileItemHbox.setSpacing(20);
            javafx.scene.control.Label fileNameLabel = new javafx.scene.control.Label(name);
            MFXProgressBar mfxProgressBar = new MFXProgressBar();
            mfxProgressBar.setProgress(0.4);
            mfxProgressBar.setPrefWidth(400);
            fileItemHbox.getChildren().addAll(fileNameLabel, mfxProgressBar);
            HBox.setHgrow(mfxProgressBar, Priority.ALWAYS);
            allSelectedFile.add(fileItemHbox);
        }
        selectedFileBox.getChildren().addAll(allSelectedFile);
    }

    public static String formatFileSize(long size) {
        if (size <= 0) return "0 B";
        final String[] units = new String[]{"B", "KB", "MB", "GB"};
        int digitGroups = (int) (Math.log10(size) / Math.log10(1024));
        return String.format("%.1f %s", size / Math.pow(1024, digitGroups), units[digitGroups]);
    }
}
