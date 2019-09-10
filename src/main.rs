use atomicwrites::{AllowOverwrite, AtomicFile};
use futures::{Future, Stream};
use mdns::{Record, RecordKind};
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep;
use std::time::Instant;
use std::{net::IpAddr, time::Duration};

/// The hostname of the devices we are searching for.
const SERVICE_NAME: &'static str = "_prometheus-http._tcp.local";

struct Service {
    labels: HashMap<String, String>,
    addr: IpAddr,
    port: u16,
    last_seen: Instant,
}

#[derive(Serialize)]
struct PrometheusService<'a> {
    targets: Vec<String>,
    labels: &'a HashMap<String, String>,
}

impl<'a> From<&'a Service> for PrometheusService<'a> {
    fn from(service: &'a Service) -> Self {
        PrometheusService {
            targets: vec![format!("{}:{}", service.addr, service.port)],
            labels: &service.labels,
        }
    }
}

const TIMEOUT: Duration = Duration::from_secs(60);
const INTERVAL: Duration = Duration::from_secs(15);

fn main() {
    let out = env::args()
        .skip(1)
        .next()
        .map(|path| AtomicFile::new(path, AllowOverwrite));

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        discover(tx);
    });

    let mut services: HashMap<IpAddr, Service> = HashMap::new();

    loop {
        let start_count = services.len();

        while let Ok(service) = rx.try_recv() {
            services.insert(service.addr, service);
        }

        let added_count = services.len();

        services.retain(|_, service| Instant::now().duration_since(service.last_seen) < TIMEOUT);

        let removed_count = services.len();

        if start_count != added_count || added_count != removed_count {
            let output_services: Vec<PrometheusService> =
                services.iter().map(|(_, service)| service.into()).collect();
            let output = serde_json::to_string(&output_services).unwrap();

            match &out {
                Some(path) => {
                    let _ = path.write(|f| f.write_all(output.as_bytes()));
                }
                None => println!("{}", output),
            }
        }

        sleep(INTERVAL);
    }
}

fn discover(tx: Sender<Service>) {
    tokio::run(
        mdns::discover::all(SERVICE_NAME, INTERVAL)
            .unwrap()
            .for_each(move |response| {
                if response
                    .records()
                    .any(|record| record.name.as_str() == SERVICE_NAME)
                {
                    let addr = response.records().filter_map(self::to_ip_addr).next();
                    let port = response.records().filter_map(self::to_port).next();
                    let labels = response.records().filter_map(self::to_labels).next();

                    if let (Some(addr), Some(labels), Some(port)) = (addr, labels, port) {
                        let _ = tx.send(Service {
                            labels,
                            addr,
                            port,
                            last_seen: Instant::now(),
                        });
                    }
                }

                Ok(())
            })
            .map_err(|e| eprintln!("{:?}", e)),
    );
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}

fn to_port(record: &Record) -> Option<u16> {
    match record.kind {
        RecordKind::SRV { port, .. } if record.name.contains(SERVICE_NAME) => Some(port),
        _ => None,
    }
}

fn to_labels(record: &Record) -> Option<HashMap<String, String>> {
    if record.name.contains(SERVICE_NAME) {
        if let RecordKind::TXT(txt) = &record.kind {
            Some(
                txt.iter()
                    .flat_map(|pair| {
                        let mut parts = pair.split('=');
                        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                            Some((key.to_string(), value.to_string()))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        } else {
            None
        }
    } else {
        None
    }
}
