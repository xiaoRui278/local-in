package wang.xiaorui.local.server;

import io.libp2p.core.PeerId;

public interface ConnectionListener {

    void onAdd(PeerId peerId, ConnectionCache connectionCache);

    void onRemove(PeerId peerId, ConnectionCache connectionCache);
}
