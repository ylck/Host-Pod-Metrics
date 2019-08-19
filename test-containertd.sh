set -x
for i in {1..5} ; do
    docker run  -d  -P -l io.kubernetes.pod.name=kube-dns-$i  -l io.kubernetes.container.name=nginx$i -l io.kubernetes.pod.namespace=test$i nginx
done

