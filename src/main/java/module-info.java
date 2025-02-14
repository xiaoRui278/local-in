module wang.xiaorui.local.localin {
    requires javax.jmdns;
    requires fr.brouillard.oss.cssfx;
    requires MaterialFX;
    requires jvm.libp2p;
    requires io.netty.buffer;

    exports wang.xiaorui.local;
    opens wang.xiaorui.local;
    opens wang.xiaorui.local.controllers;
}