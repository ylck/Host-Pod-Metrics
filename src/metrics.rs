use prometheus::*;

lazy_static::lazy_static! {
    pub static ref POD_INFO: IntGaugeVec = register_int_gauge_vec!(
        "A_pod_host_info",
        "kube pod in host",
        &[
            "c_name",
            "c_podname",
            "c_podnamenamespace",
            "h_name",
            "h_ip",
            "c_pid",
            "c_nlink",

        ]
    )
    .unwrap();
}
