use atomicwrites::{AllowOverwrite, AtomicFile};
use futures_util::{pin_mut, stream::StreamExt};
use mdns::Response;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::net::SocketAddr;
use std::time::Duration;
use std::time::Instant;

/// The hostname of the devices we are searching for.
const SERVICE_NAME: &str = "_prometheus-http._tcp.local";

struct Service {
    labels: HashMap<String, String>,
    addr: SocketAddr,
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
            targets: vec![service.addr.to_string()],
            labels: &service.labels,
        }
    }
}

const TIMEOUT: Duration = Duration::from_secs(360);
const INTERVAL: Duration = Duration::from_secs(120);

#[tokio::main]
async fn main() -> Result<(), main_error::MainError> {
    let out = env::args()
        .nth(1)
        .map(|path| AtomicFile::new(path, AllowOverwrite));

    let stream = mdns::discover::all(SERVICE_NAME, INTERVAL)?.listen();
    pin_mut!(stream);

    let mut services: HashMap<SocketAddr, Service> = HashMap::new();

    while let Some(Ok(response)) = stream.next().await {
        let response: Response = response;
        let addr = response.socket_address();
        let mut labels: HashMap<String, String> = response
            .txt_records()
            .flat_map(|pair| {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    Some((key.to_string(), value.to_string()))
                } else {
                    None
                }
            })
            .collect();
        let hostname = response
            .hostname()
            .and_then(|host| host.split('.').next().map(|s| s.to_string()));

        if let (Some(addr), Some(hostname)) = (addr, hostname) {
            labels.insert("hostname".to_string(), hostname.to_string());
            let service = Service {
                labels,
                addr,
                last_seen: Instant::now(),
            };

            let start_count = services.len();
            services.insert(service.addr, service);

            let added_count = services.len();

            services
                .retain(|_, service| Instant::now().duration_since(service.last_seen) < TIMEOUT);

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
        }
    }

    Ok(())
}
