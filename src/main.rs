use rs_docker::Docker;
use serde_json::json;
use std::collections::{HashMap, VecDeque};

///"Labels": {
///   "io.cri-containerd.kind": "container",
///    "io.kubernetes.container.name": "dnsmasq",
///    !="io.kubernetes.container.name": "POD",
///    "io.kubernetes.pod.name": "kube-dns-6bfbdd666c-5jbmx",
///    "io.kubernetes.pod.namespace": "kube-system",
///    "io.kubernetes.pod.uid": "5528e13d-5df8-11e9-a377-001c427c953a"
///    },
///
#[derive(Copy, Clone, Debug)]
pub struct PodContainers<'a> {
    c_id: &'a str,
    c_name: &'a str,
    c_podname: &'a str,
    c_podnamenamespace: &'a str,
    c_pid: &'a str,
}

fn main() {
    let mut docker = match Docker::connect("unix:///var/run/docker.sock") {
        Ok(docker) => docker,
        Err(e) => {
            panic!("{}", e);
        }
    };
    let containers = match docker.get_containers(false) {
        Ok(containers) => containers,
        Err(e) => {
            panic!("{}", e);
        }
    };

    // 构建容器ID队列
    let mut podcs = VecDeque::new();

    for c in &containers {
        let john = json!(c.Labels);
        if john["io.kubernetes.pod.name"] != ""&&john["io.kubernetes.container.name"] != "POD"  {
            podcs.push_back(c);
        }

    }
    //获取容器进程在 host PID
    for c in &podcs {
        let c_top = match docker.get_processes(&c) {
            Ok(c_top) => c_top,
            Err(e) => {
                panic!("{}", e);
            }
        };
        println!("{}", c_top[0].pid);

        let mut c_data: HashMap<String, PodContainers> = HashMap::new();

        let mut l = &HashMap::new();
        let mut c_name = "";
        let mut c_pname = "";
        let mut c_pnames = "";
        if let Some(x) = &c.Labels {
            l = x;
        }

        if let Some(x) = l.get("io.kubernetes.container.name") {
            c_name = x;
        }
        if let Some(x) = l.get("io.kubernetes.pod.name") {
            c_pname = x;
        }
        if let Some(x) = l.get("io.kubernetes.pod.namespace") {
            c_pnames = x;
        }

        c_data.insert(
            c.Id.to_string(),
            *&PodContainers {
                c_id: c.Id.as_str(),
                c_name: c_name,
                c_podname: c_pname,
                c_podnamenamespace: c_pnames,
                c_pid: c_top[0].pid.as_str(),
            },
        );

        println!(
            "docker info: {:?}",
            c_data.get(&c.Id.to_string())
        );
    }

}
