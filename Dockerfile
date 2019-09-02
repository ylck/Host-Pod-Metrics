FROM debian:buster-slim
RUN sed -i 's/deb.debian.org/mirrors.ustc.edu.cn/g' /etc/apt/sources.list && apt update && apt install net-tools iproute2 -y && apt clean
COPY target/x86_64-unknown-linux-musl/release/pod_info /
CMD ["/pod_info"]
