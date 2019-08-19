FROM scratch
COPY target/x86_64-unknown-linux-musl/release/pod_info /
CMD ["/pod_info"]