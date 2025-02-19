package wang.xiaorui.local.p2p.listener;

import io.libp2p.core.Host;
import io.libp2p.core.PeerId;
import io.libp2p.core.PeerInfo;
import io.libp2p.core.multiformats.Multiaddr;
import io.libp2p.discovery.mdns.AnswerListener;
import io.libp2p.discovery.mdns.impl.DNSRecord;
import io.libp2p.discovery.mdns.impl.constants.DNSRecordType;
import wang.xiaorui.local.p2p.founder.P2PPeerFounder;

import java.util.ArrayList;
import java.util.List;

/**
 * @author wangxiaorui
 * @date 2025/2/12
 * @desc
 */
public class P2PAnswerListener implements AnswerListener {

    private final Host host;

    private List<P2PPeerFounder> peerFounders = new ArrayList<>();

    public P2PAnswerListener(Host host) {
        this.host = host;
    }

    public void addPeerFounder(P2PPeerFounder peerFounder) {
        peerFounders.add(peerFounder);
    }

    @Override
    public void answersReceived(List<DNSRecord> answers) {
        DNSRecord txtRecordS = answers.stream()
                .filter(answer -> DNSRecordType.TYPE_TXT.equals(answer.getRecordType()))
                .findFirst().orElse(null);

        DNSRecord srvRecordS = answers.stream()
                .filter(answer -> DNSRecordType.TYPE_SRV.equals(answer.getRecordType()))
                .findFirst().orElse(null);

        List<DNSRecord> aRecordsS = answers.stream()
                .filter(answer -> DNSRecordType.TYPE_A.equals(answer.getRecordType()))
                .toList();
        List<DNSRecord> aaaaRecordsS = answers.stream()
                .filter(answer -> DNSRecordType.TYPE_AAAA.equals(answer.getRecordType()))
                .toList();
        if (null == txtRecordS || null == srvRecordS ||
                (aRecordsS.isEmpty() && aaaaRecordsS.isEmpty())) {
            return;
        }
        DNSRecord.Text txtRecord = (DNSRecord.Text) txtRecordS;
        DNSRecord.Service srvRecord = (DNSRecord.Service) srvRecordS;
        String peerIdStr = new String(txtRecord.getText());
        if (peerIdStr.startsWith(".")) {
            peerIdStr = peerIdStr.substring(1);
        }
        PeerId peerId = PeerId.fromBase58(peerIdStr);
        int port = srvRecord.getPort();
        List<Multiaddr> multiAddrs = new ArrayList<>();
        List<String> hostAddress = new ArrayList<>();
        if (!aRecordsS.isEmpty()) {
            aRecordsS.forEach(it -> {
                DNSRecord.IPv4Address iPv4Address = (DNSRecord.IPv4Address) it;
                String ipv4 = iPv4Address.getAddress().getHostAddress();
                multiAddrs.add(new Multiaddr("/ip4/" + ipv4 + "/tcp/" + port));
                hostAddress.add(ipv4);
            });
        }
        if (!aaaaRecordsS.isEmpty()) {
            aaaaRecordsS.forEach(it -> {
                DNSRecord.IPv6Address iPv6Address = (DNSRecord.IPv6Address) it;
                String ipv6 = iPv6Address.getAddress().getHostAddress();
                multiAddrs.add(new Multiaddr("/ip6/" + ipv6 + "/tcp/" + port));
                hostAddress.add(ipv6);
            });
        }
        PeerInfo peerInfo = new PeerInfo(peerId, multiAddrs);
        //调用节点通知接口
        peerFounders.forEach(peerFounder -> peerFounder.peerFound(host, peerInfo, hostAddress));
    }
}
