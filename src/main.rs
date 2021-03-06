use crossbeam_channel::bounded;
//use crossbeam_utils::thread;
use actix_web::{ web, App, HttpResponse, HttpServer, Result,Responder};

use chrono;

use prometheus::*;

use rs_docker::Docker;
use serde_json::json;
use std::collections::{HashMap, VecDeque};

use std::thread;

use sys_info;
mod metrics;
use self::metrics::*;

use regex::Regex;
use std::borrow::Borrow;
use std::process::Command;

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
    c_nlink: &'a str,
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
    let sys = actix_rt::System::new("pod-info");
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/metrics").route(web::get().to(metrics)))
            .default_service(web::resource("").route(web::get().to(p404)))
    })
        .bind("0.0.0.0:8088")?
        .shutdown_timeout(15)
        .start();


    println!("Starting http server: http://{}","0.0.0.0:8088");
    sys.run()
}
/// 404 handler
fn p404() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body("<a href=/metrics>IP</a>".to_string()))
}

fn metrics() -> impl Responder {
    //    POD_INFO
    //        .with_label_values(&["1", "b", "dd", "sdv", "ddcc"])
    //        .set(10);
    //    POD_INFO
    //        .with_label_values(&["ddvdvdv", "bvddvdv", "dd", "sdv", "ddcc"])
    //        .set(10);
    let mut contacts: HashMap<String, String> = HashMap::new();

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

    let (s, r) = bounded(1); // Make room for one unmatched send.
    thread::spawn(move || {
        // This call blocks the current thread until a receive operation appears
        // on the other side of the channel.
        let output = Command::new("ss").arg("-tunp4").output().expect("no shell");
        if output.status.success() {
            s.send(
                String::from_utf8_lossy(&output.stdout)
                    .trim_end()
                    .to_string(),
            )
            .unwrap();
        }
    });
    if let Ok(out) = r.recv() {
        let split = out.split("tcp");
        for s in split {
            //                println!("{:?}", s);
            let re = Regex::new(
                r"(?:(?:[0,1]?\d?\d|2[0-4]\d|25[0-5])\.){3}(?:[0,1]?\d?\d|2[0-4]\d|25[0-5]):\d{0,5}",
            )
                .unwrap();
            let pid = Regex::new(r"pid=\d{0,5}").unwrap();

            let text = s.clone();
            for pid in pid.captures_iter(text) {
                println!("{}", &pid[0].replace("pid=", ""));

                //                for address in re.captures_iter(text) {
                //                    for i in address {
                //                        println!("{}", i);
                //                    }
                //
                //                    let addr: String = address[0].parse().unwrap();
                //                    contacts.insert(pid[0].replace("pid=", ""), addr);
                //                    //                    let addr: String = address[1].parse().unwrap();
                //                    //                    contacts2.insert(pid[0].replace("pid=", ""), addr);
                //                }
                let caps = re.captures(text).unwrap();
                let nlink = caps.get(0).map_or("", |m| m.as_str());

                contacts.insert(pid[0].replace("pid=", ""), nlink.to_string());
            }
        }
    } else {
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
        let contacts = contacts.clone();
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
            match r.recv() {
                Ok(cc_top) => {
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
                            c_nlink: "",
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
                            contacts
                                .get(cc_top[0].pid.as_str())
                                .unwrap_or("0000".to_string().borrow())
                                .as_str(),
                        ])
                        .set(0);

                    //                }

                    println!("docker info: {:?}", &c_data.get(&p.Id.to_string()));
                }
                Err(..) => {}
            }
        });
    }

    let metric_families = prometheus::gather();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    println!("{:?}", contacts);

    String::from_utf8(buffer).unwrap()
}
