## 新书推荐服务

### 启动服务

```sh
# 按照此格式创建配置： { "auth": { "user": "xxx@qq.com", "pass": "xxxx" }, "to": "xxx@qq.com" }
touch .env.json
# 编译可执行文件
cargo build --release
# 运行后台服务
pm2 start target/release/book-weekly
```

### 相关阅读

选择 http client 库： https://blog.logrocket.com/the-state-of-rust-http-clients/
解析 json ：https://json.im/jichu/rust-json.html
https://mirrors.segmentfault.com/rust/README_3.html
https://www.ixiqin.com/2019/02/macos-installation-rust-development-environment/
https://doc.rust-lang.org/std/process/struct.Command.html
