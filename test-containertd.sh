version: '3.7'
services:
  web:
    images: nginx
#    labels:
#      io.cri-containerd.kind: "container",
#      io.kubernetes.container.name: "dnsmasq",
#      io.kubernetes.pod.name: "kube-dns-6bfbdd666c-5jbmx",
#      io.kubernetes.pod.namespace: "kube-system",
#      io.kubernetes.pod.uid: "5528e13d-5df8-11e9-a377-001c427c953a",
    ports:
      - "5000:80"