# local-in

#### 介绍
Local-In 是一款基于 JavaFX 和 libp2p 协议开发的点对点（P2P）局域网聊天应用。该应用实现了用户之间的直接消息传输，无需经过任何中心服务器，确保了数据的安全性和隐私性。

#### 软件架构
JDK版本：`17+`  
UI框架：`JavaFX + MaterialFX`  
P2P组件：`libp2p(jvm-libp2p)`  

#### 安装教程
1. 下载源码：访问项目仓库并下载最新源码。
2. 导入 IDE：将源码导入您偏好的集成开发环境（IDE）。
3. 运行应用：执行 `LocalInApplication` 类以启动应用

#### 使用说明
1. 启动应用：运行 `LocalInApplication` 启动聊天应用。
2. 加入聊天室：输入或选择要加入的聊天室。
3. 发送消息：在聊天界面输入消息并发送。

#### 参与贡献
1.  Fork仓库：在github上fork本项目
2.  新建分支：基于 `master` 分支创建新分支，命名为 `feature/xxxx`
3.  提交代码：在新分支上进行代码修改并提交`feature/develop`
4.  Pull Request：创建 Pull Request 请求合并代码
5.  等待合并：等待管理员合并您的代码

#### 分支说明
本仓库包含两个主要分支：`master` 和 `feature/develop`。
`master`分支：该分支包含所有已合并的 Pull Request，并定期发布正式版本。
`feature/develop`分支：该分支包含所有未发版的 Pull Request，用于开发新功能和修复 bug。

#### 软件截图
在线用户
![软件截图](/docs/images/online.png)
![软件截图](/docs/images/onlineUser.png)
