package wang.xiaorui.local;

import fr.brouillard.oss.cssfx.CSSFX;
import io.github.palexdev.materialfx.theming.JavaFXThemes;
import io.github.palexdev.materialfx.theming.MaterialFXStylesheets;
import io.github.palexdev.materialfx.theming.UserAgentBuilder;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import wang.xiaorui.local.controllers.LocalInController;
import wang.xiaorui.local.p2p.P2PServer;
import wang.xiaorui.local.server.LocalInP2PServer;

import java.io.IOException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public class LocalInApplication extends Application {

    private final ExecutorService executorService = Executors.newCachedThreadPool();

    private P2PServer p2PServer;

    @Override
    public void start(Stage primaryStage) throws IOException {
        //启动CSSFX,允许程序动态修改CSS样式
        CSSFX.start();
        //主题设置
        UserAgentBuilder.builder()
                .themes(JavaFXThemes.MODENA)
                .themes(MaterialFXStylesheets.forAssemble(true))
                .setDeploy(true)
                .setResolveAssets(true)
                .build()
                .setGlobal();
        //
        //主界面
        FXMLLoader loader = new FXMLLoader(MFXDemoResourcesLoader.loadURL("fxmls/LocalIn.fxml"));
        loader.setControllerFactory(c -> new LocalInController(primaryStage));
        Parent root = loader.load();

        Scene scene = new Scene(root);
        scene.setFill(Color.TRANSPARENT);
        primaryStage.initStyle(StageStyle.TRANSPARENT);
        primaryStage.setScene(scene);
        primaryStage.setTitle("Local In");
        primaryStage.show();

        startP2P();
        //Platform.runLater(this::registerService);
    }

    private void startP2P(){
        try {
            //启动P2P服务
            p2PServer = new LocalInP2PServer();
            p2PServer.start();
        } catch (Exception e) {
            e.printStackTrace();
            throw new RuntimeException(e);
        }
    }


    @Override
    public void stop() throws Exception {
        System.out.println("=======stop=======");
        if (p2PServer != null) {
            p2PServer.stop();
        }
        // 关闭线程池并等待所有任务完成
        executorService.shutdown();
        super.stop();
        Platform.exit();
    }

    public static void main(String[] args) {
        launch();
    }
}