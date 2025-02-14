package wang.xiaorui.local.utils;

import java.io.IOException;
import java.net.*;
import java.util.Enumeration;

/**
 * @author wangxiaorui
 * @date 2025/2/8
 * @desc
 */
public class NetworkUtil {

    public static String getMacAddress() {
        try {
            Enumeration<NetworkInterface> networkInterfaces = NetworkInterface.getNetworkInterfaces();
            while (networkInterfaces.hasMoreElements()) {
                NetworkInterface networkInterface = networkInterfaces.nextElement();
                byte[] mac = networkInterface.getHardwareAddress();
                if (mac != null) {
                    StringBuilder sb = new StringBuilder();
                    for (int i = 0; i < mac.length; i++) {
                        sb.append(String.format("%02X%s", mac[i], (i < mac.length - 1) ? "-" : ""));
                    }
                    return sb.toString();
                }
            }
        } catch (SocketException e) {
            e.printStackTrace();
        }
        return null;
    }

    public static int findAvailablePort(int startPort, int endPort) {
        for (int port = startPort; port <= endPort; port++) {
            try (ServerSocket serverSocket = new ServerSocket(port)) {
                return port;
            } catch (IOException e) {
                // Port is not available, continue to the next port
            }
        }
        throw new RuntimeException("No available port found in the range: " + startPort + " to " + endPort);
    }

    public static boolean isPortOpen(InetAddress address, int port) {
        try (Socket socket = new Socket()) {
            socket.connect(new java.net.InetSocketAddress(address, port), 1000);
            return true;
        } catch (IOException e) {
            return false;
        }
    }

    public static boolean ipOrPortIsChanged(String oldIp, int oldPort, String newIp, int newPort) {
        return !oldIp.equals(newIp) || oldPort != newPort;
    }

    /**
     * 获取局域网IP地址
     *
     * @return 局域网IP地址
     * @throws SocketException
     */
    public static InetAddress getLocalInetAddress() throws SocketException {
        Enumeration<NetworkInterface> networkInterfaces = NetworkInterface.getNetworkInterfaces();
        while (networkInterfaces.hasMoreElements()) {
            NetworkInterface ni = networkInterfaces.nextElement();
            Enumeration<InetAddress> inetAddresses = ni.getInetAddresses();
            while (inetAddresses.hasMoreElements()) {
                InetAddress ia = inetAddresses.nextElement();
                if (!ia.isLoopbackAddress() && ia.isSiteLocalAddress()) {
                    return ia;
                }
            }
        }
        throw new RuntimeException("No suitable network interface found.");
    }
}
