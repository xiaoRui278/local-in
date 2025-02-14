package wang.xiaorui.local.p2p.founder;

import io.libp2p.core.Host;
import io.libp2p.core.PeerInfo;

import java.util.List;

public interface P2PPeerFounder {

    void peerFound(Host host, PeerInfo peerInfo, List<String> hostAddress);
}
