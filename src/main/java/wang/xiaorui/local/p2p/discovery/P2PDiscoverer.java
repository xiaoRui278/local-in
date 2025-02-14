package wang.xiaorui.local.p2p.discovery;

import io.libp2p.core.Host;
import io.libp2p.core.multiformats.Multiaddr;
import io.libp2p.core.multiformats.MultiaddrComponent;
import io.libp2p.core.multiformats.Protocol;
import io.libp2p.discovery.mdns.JmDNS;
import io.libp2p.discovery.mdns.ServiceInfo;
import wang.xiaorui.local.constants.Constants;
import wang.xiaorui.local.p2p.founder.P2PPeerFounder;
import wang.xiaorui.local.p2p.listener.P2PAnswerListener;
import wang.xiaorui.local.utils.NetworkUtil;

import java.io.IOException;
import java.net.*;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.stream.Stream;

/**
 * @author wangxiaorui
 * @date 2025/2/12
 * @desc P2P节点发现器
 */
public class P2PDiscoverer {

    private JmDNS jmDNS;

    private final Host host;

    private final InetAddress localhost = NetworkUtil.getLocalInetAddress();

    private final P2PAnswerListener answerListener;


    public P2PDiscoverer(Host host) throws IOException {
        this.host = host;
        this.answerListener = new P2PAnswerListener(host);
    }

    public void addPeerFounder(P2PPeerFounder peerFounder) {
        this.answerListener.addPeerFounder(peerFounder);
    }

    /**
     * 启动服务发现
     */
    public void start() throws IOException {
        //本机地址启动JmDNS
        jmDNS = JmDNS.create(localhost);
        //注册本机服务
        jmDNS.registerService(serviceInfo());
        //添加监听
        jmDNS.addAnswerListener(Constants.SERVICE_TYPE, 1000, answerListener);
        jmDNS.start();
        System.out.println("P2P Discoverer started");
    }

    /**
     * 停止服务发现
     */
    public void stop() {
        System.out.println("P2P Discoverer stopped");
        jmDNS.stop();
    }

    private ServiceInfo serviceInfo() {
        return ServiceInfo.create(
                Constants.SERVICE_TYPE,
                host.getPeerId().toBase58(),
                listenPort(),
                host.getPeerId().toBase58(),
                ipAddresses(Protocol.IP4, Inet4Address.class),
                ipAddresses(Protocol.IP6, Inet6Address.class));
    }

    private int listenPort() {
        List<Multiaddr> listenAddresses = host.listenAddresses();

        Multiaddr address = listenAddresses.stream()
                .filter(a -> a.has(Protocol.IP4))
                .findFirst()
                .orElse(null);

        Multiaddr ipv6OnlyAddress = (address == null)
                ? listenAddresses.stream()
                .filter(a -> a.has(Protocol.IP6))
                .findFirst()
                .orElse(null)
                : address;

        if (ipv6OnlyAddress == null) {
            throw new IllegalStateException("No valid listen address found");
        }

        String str = ipv6OnlyAddress.getFirstComponent(Protocol.TCP).getStringValue();
        return Integer.parseInt(str);
    }

    private <T> List<T> ipAddresses(Protocol protocol, Class<T> clazz) {
        return host.listenAddresses().stream()
                .flatMap(it -> this.expandWildcardAddresses(it).stream())
                .map(it -> it.getFirstComponent(protocol))
                .filter(Objects::nonNull)
                .map(it -> {
                    try {
                        return InetAddress.getByAddress(localhost.getHostName(), it.getValue());
                    } catch (UnknownHostException e) {
                        throw new RuntimeException(e);
                    }
                })
                .filter(clazz::isInstance)
                .map(clazz::cast)
                .toList();


    }

    private List<Multiaddr> expandWildcardAddresses(Multiaddr addr) {
        // Do not include /p2p or /ipfs components which are superfluous here
        if (!isWildcard(addr)) {
            return java.util.List.of(
                    new Multiaddr(
                            addr.getComponents()
                                    .stream()
                                    .filter(c -> c.getProtocol() != Protocol.P2P &&
                                            c.getProtocol() != Protocol.IPFS)
                                    .toList()
                    )
            );
        }
        if (addr.has(Protocol.IP4)) return listNetworkAddresses(false, addr);
        if (addr.has(Protocol.IP6)) return listNetworkAddresses(true, addr);
        else return Collections.emptyList();
    }

    private List<Multiaddr> listNetworkAddresses(Boolean includeIp6, Multiaddr addr) {
        try {
            return Collections.list(NetworkInterface.getNetworkInterfaces()).stream()
                    .flatMap(net -> net.getInterfaceAddresses().stream()
                            .map(InterfaceAddress::getAddress)
                            .filter(ip -> includeIp6 || ip instanceof Inet4Address)
                    ).map(ip ->
                            new Multiaddr(
                                    Stream.concat(
                                            Stream.of(
                                                    new MultiaddrComponent(
                                                            ip instanceof Inet4Address ? Protocol.IP4 : Protocol.IP6,
                                                            ip.getAddress()
                                                    )
                                            ),
                                            addr.getComponents().stream()
                                                    .filter(c -> c.getProtocol() != Protocol.IP4 && c.getProtocol() != Protocol.IP6 && c.getProtocol() !=
                                                            Protocol.P2P && c.getProtocol() != Protocol.IPFS)
                                    ).toList()
                            )).toList();
        } catch (SocketException e) {
            throw new RuntimeException(e);
        }
    }

    private Boolean isWildcard(Multiaddr addr) {
        String s = addr.toString();
        return s.contains("/::/") || s.contains("/0:0:0:0/");
    }
}
