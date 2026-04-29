# 第一阶段：编译（使用 Alpine 以生成 musl 静态二进制）
FROM rust:alpine as builder

WORKDIR /usr/src/app

# 安装 musl 开发工具（静态链接必需）
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

# 复制 Cargo 文件并缓存依赖
COPY Cargo.toml Cargo.lock* ./
# 复制源代码并编译真实二进制
COPY src ./src
RUN cargo build --release

# 第二阶段：运行时（极轻量）
FROM alpine

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/hookline /app/hookline

EXPOSE 8080

CMD ["/app/hookline"]

