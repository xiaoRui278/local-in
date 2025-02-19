package wang.xiaorui.local.server;

import io.libp2p.core.Stream;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import wang.xiaorui.local.handler.PersonalMessageHandler;
import wang.xiaorui.local.p2p.message.P2PAbstractMessageHandler;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/**
 * @author wangxiaorui
 * @date 2025/2/13
 * @desc
 */
public class LocalInP2PMessageHandler extends P2PAbstractMessageHandler {
    private final ConnectionCache connectionCache;
    private List<LocalInMessageObserver> messageObservers = new ArrayList<>();

    public LocalInP2PMessageHandler(Stream stream) {
        super(stream);
        this.connectionCache = ConnectionCache.getInstance();
    }

    public void addMessageObserver(LocalInMessageObserver messageObservers) {
        this.messageObservers.add(messageObservers);
    }

    @Override
    public void onActivated(Stream stream) {
        System.out.println("---LocalInP2PMessageHandler>>>onActivated[" + stream.remotePeerId().toBase58() + "]----->");
    }

    @Override
    public void onClosed(Stream stream) {
        System.out.println("---LocalInP2PMessageHandler>>>onClosed[" + stream.remotePeerId().toBase58() + "]----->");
        connectionCache.removePeer(stream.remotePeerId());
    }

    @Override
    public void onException(Throwable cause) {
        System.out.println("---LocalInP2PMessageHandler>>>onException[" + stream.remotePeerId().toBase58() + "]----->");
    }

    @Override
    public void onMessage(Stream stream, ByteBuf msg) {
        String fromUser = stream.remotePeerId().toBase58();
        System.out.println("---LocalInP2PMessageHandler>>>onMessage[" + fromUser + "]----->");
        String message = msg.toString(StandardCharsets.UTF_8);
        System.out.println("---LocalInP2PMessageHandler>>>onMessage---[" + message + "]-->");
        if (null == message || message.isEmpty()) {
            return;
        }
        if (message.startsWith("/group")) {
            //群发消息
            //去掉群发消息前缀
            message = message.substring("/group".length());
            //渲染消息
            for (LocalInMessageObserver messageObserver : messageObservers) {
                messageObserver.onMessage(message);
            }
            return;
        }
        if(message.startsWith("/personal")){
            message = message.substring("/personal".length());
            int splitIndex = message.indexOf("/");
            String toUser = message.substring(0, splitIndex + 1);
            String text = message.substring(splitIndex + 1);
            PersonalMessageHandler.getInstance().onMessage(fromUser, text);
        }
    }

    @Override
    public void send(String message) {
        System.out.println("---LocalInP2PMessageHandler>>>send[" + stream.remotePeerId().toBase58() + "]----->");
        byte[] bytes = message.getBytes(StandardCharsets.UTF_8);
        ByteBuf messageBuf = Unpooled.wrappedBuffer(bytes);// Unpooled.copiedBuffer(message, StandardCharsets.UTF_8);
        stream.writeAndFlush(messageBuf);
    }
}
