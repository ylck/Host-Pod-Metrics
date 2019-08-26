use crossbeam_channel::bounded;
//use crossbeam_utils::thread;
use actix_web::{web, App, HttpServer, Responder};
use chrono;

use prometheus::*;

use rs_docker::Docker;
use serde_json::json;
use std::collections::{HashMap, VecDeque};

use std::thread;

use sys_info;
mod metrics;
use self::metrics::*;

extern crate fern;

extern crate log;

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
    h_name: &'a str,
    h_ip: &'a str,
}

#[derive(Copy, Clone, Debug)]
pub struct PidNetlink<'a> {
    //TCP UDP
    net_id: &'a str,
    //Local Address:Port
    l_ap: &'a str,
    //Peer Address:Port
    p_ap: &'a str,
}

fn main() -> std::io::Result<()> {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        // - and per-module overrides
        .level_for("hyper", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        // Apply globally
        .apply();
    //    loop {
    //        let (s, r) = bounded(1); // Make room for one unmatched send.
    //        thread::spawn(move || {
    //            // This call blocks the current thread until a receive operation appears
    //            // on the other side of the channel.
    //            let output = Command::new("ss").arg("-tunp4").output().expect("no shell");
    //            if output.status.success() {
    //                s.send(
    //                    String::from_utf8_lossy(&output.stdout)
    //                        .trim_end()
    //                        .to_string(),
    //                )
    //                .unwrap();
    //            }
    //        });
    //        if let Ok(out) = r.recv() {
    //            let split = out.split("tcp");
    //            for s in split {
    //                //                println!("{:?}", s);
    //                let re = Regex::new(
    //                    r"(?:(?:[0,1]?\d?\d|2[0-4]\d|25[0-5])\.){3}(?:[0,1]?\d?\d|2[0-4]\d|25[0-5]):\d{0,5}",
    //                )
    //                    .unwrap();
    //                let re2 = Regex::new(r"pid=\d{0,5}").unwrap();
    //
    //                let text = s.clone();
    //                for cap in re.captures_iter(text) {
    //                    println!("{}", &cap[0]);
    //                }
    //                for cap in re2.captures_iter(text) {
    //                    println!("{}", &cap[0]);
    //                }
    //            }
    //        }
    //
    //        thread::sleep(Duration::from_secs(1));
    //    }

    HttpServer::new(move || App::new().service(web::resource("/metrics").to(metrics)))
        .bind("0.0.0.0:8088")?
        .run()
}

fn metrics() -> impl Responder {
    //    POD_INFO
    //        .with_label_values(&["1", "b", "dd", "sdv", "ddcc"])
    //        .set(10);
    //    POD_INFO
    //        .with_label_values(&["ddvdvdv", "bvddvdv", "dd", "sdv", "ddcc"])
    //        .set(10);
    let hostname = sys_info::hostname().unwrap();
    let ip = local_ip::get().unwrap().to_string();
    println!("local ip address: {:?}", ip);
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

    for c in containers {
        let john = json!(c.Labels);
        if john["io.kubernetes.pod.name"] != "" && john["io.kubernetes.container.name"] != "POD" {
            podcs.push_back(c);
        }
    }
    //获取容器进程在 host PID
    for p in podcs {
        let (s, r) = bounded(10);
        let p = p.clone();
        let mut docker = match Docker::connect("unix:///var/run/docker.sock") {
            Ok(docker) => docker,
            Err(e) => {
                panic!("{}", e);
            }
        };
        let c_data: HashMap<String, PodContainers> = HashMap::new();
        let hostname = hostname.clone();
        let ip = ip.clone();

        thread::spawn(move || {
            let c_top = match docker.get_processes(&p) {
                Ok(c_top) => c_top,
                Err(e) => {
                    panic!("{}", e);
                }
            };
            println!("{}", c_top[0].pid);
            s.send(c_top).unwrap();
            let mut l = &HashMap::new();
            let mut c_name = "";
            let mut c_pname = "";
            let mut c_pnames = "";
            if let Some(x) = &p.Labels {
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
            let mut c_data = c_data.clone();
            if let Ok(cc_top) = r.recv() {
                println!("{}", cc_top[0].pid.as_str());
                c_data.insert(
                    p.Id.to_string(),
                    PodContainers {
                        c_id: p.Id.as_str(),
                        c_name: c_name,
                        c_podname: c_pname,
                        c_podnamenamespace: c_pnames,
                        c_pid: cc_top[0].pid.as_str(),
                        h_name: &hostname.as_str(),
                        h_ip: &ip.as_str(),
                    },
                );
                POD_INFO
                    .with_label_values(&[
                        c_name,
                        c_pname,
                        c_pnames,
                        &hostname.as_str(),
                        &ip.as_str(),
                        cc_top[0].pid.as_str(),
                    ])
                    .set(cc_top[0].pid.parse().unwrap());
                println!("docker info: {:?}", &c_data.get(&p.Id.to_string()));
            }
        });
    }

    let metric_families = prometheus::gather();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
}
