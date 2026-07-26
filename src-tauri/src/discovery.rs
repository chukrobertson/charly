use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PeerInfo {
    pub ip: String,
    pub port: u16,
}

pub fn check_peer_otg(ip: &str, port: u16) -> Result<bool, String> {
    let url = format!("http://{}:{}/api/health", ip, port);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .map_err(|e| e.to_string())?;
    match client.get(&url).send() {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),
    }
}

pub fn get_tailscale_peers() -> Vec<String> {
    let output = std::process::Command::new("tailscale")
        .args(["status", "--json"])
        .output()
        .ok();
    if let Some(out) = output {
        if out.status.success() {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                let mut ips = Vec::new();
                if let Some(peers) = v.get("Peer").and_then(|p| p.as_object()) {
                    for (_key, peer) in peers {
                        if let Some(ips_arr) = peer.get("TailscaleIPs").and_then(|i| i.as_array()) {
                            for ip in ips_arr {
                                if let Some(ip_str) = ip.as_str() {
                                    ips.push(ip_str.to_string());
                                }
                            }
                        }
                    }
                }
                return ips;
            }
        }
    }
    Vec::new()
}
