# 测试是否配置好 .env.json
cargo test test_read_config -- --nocapture
# 编译可执行文件
cargo build --release
# 运行后台服务
pm2 start target/release/book-weekly