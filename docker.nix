{
  dockerTools,
  prometheus-mdns-sd,
}:
dockerTools.buildLayeredImage {
  name = "icewind1991/prometheus-mdns-sd";
  tag = "latest";
  maxLayers = 5;
  contents = [
    prometheus-mdns-sd
    dockerTools.caCertificates
  ];
  config = {
    Cmd = ["prometheus-mdns-sd-rs"];
  };
}
