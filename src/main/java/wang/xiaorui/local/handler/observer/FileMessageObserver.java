package wang.xiaorui.local.handler.observer;

import io.libp2p.core.PeerId;

public interface FileMessageObserver {

    /**
     * 收到文件Meta消息
     */
    void onAcceptFileMetaMessage(PeerId peerId, String fileName, String fileSize);
}
