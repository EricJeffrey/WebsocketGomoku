# Rust练习项目 - 五子棋程序

## 目录

- WebsockEnd: rust+websocket编写的后端，类似于TCP连接，使用了[websocket-rs](http://websockets-rs.github.io/rust-websocket/doc/websocket/index.html).
- frontend: react+[pixi_js](https://www.pixijs.com/)的前端，略粗糙

## 构建运行

### 运行
- websocket: 进入WebsockEnd目录并执行`cargo run`
- frontend: 进入frontend目录并执行`npm run start`

### 构建
- websocket: 进入WebsockEnd目录并执行`cargo build --release`，得到的是二进制程序，可直接在Ubuntu等系统上运行
- frontend: 进入frontend目录并执行`npm run build`
