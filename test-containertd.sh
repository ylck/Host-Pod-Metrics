docker run  -d  -P -l io.kubernetes.pod.name=kube-dns-1 -l io.kubernetes.container.name=nginx -l io.kubernetes.pod.namespace=test nginx
docker run  -d  -P -l io.kubernetes.pod.name=kube-dns-2 -l io.kubernetes.container.name=nginx1 -l io.kubernetes.pod.namespace=test nginx
docker run  -d  -P -l io.kubernetes.pod.name=kube-dns-3 -l io.kubernetes.container.name=nginx2 -l io.kubernetes.pod.namespace=test nginx
