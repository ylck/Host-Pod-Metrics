#! /bin/bash
rm -f pod_info
pkill pod_info
wget http://sc.ylck.me:8000/target/x86_64-unknown-linux-musl/release/pod_info
chmod +x pod_info
nohup ./pod_info &
