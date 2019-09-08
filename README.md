# mDNS service discovery for Prometheus

Discovers mDNS/ZeroConf/Bonjour service announcements under _prometheus-http._tcp for ad-hoc discovery of devices on LAN networks.

## Usage

Run the service discovery daemon

```
prometheus-mdns-sd-rs /etc/prometheus/mdns-sd.json
```

Configure prometheus to use the output file

```yaml
- job_name: mdns-sd
  scrape_interval: 30s
  scrape_timeout: 10s
  metrics_path: /metrics
  scheme: http
  file_sd_configs:
  - files:
    - /etc/prometheus/mdns-sd.json
    refresh_interval: 5m
```

## Advertising services

An example of advertising services using the esp8266 arduino sdk

```arduino
if (!MDNS.begin(hostString)) {
    Serial.println("Error setting up MDNS responder!");
}
MDNS.addService("prometheus-http", "tcp", 80);
MDNS.addServiceTxt("prometheus-http", "tcp", "name", prometheus_name);
```

## Credits

Wholly inspired by [prometheus-mdns-sd](https://github.com/msiebuhr/prometheus-mdns-sd) by Morten Siebuhr  