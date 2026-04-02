use anyhow::Result;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

/// test latency of a DNS server by sending a minimal query
///
/// # args
/// * `dns_ip` - Indirizzo IP del server DNS (es. "8.8.8.8")
///
/// # returns
/// * `Ok(ms)` - latency in millis
/// * `Err(_)` - if timeout
pub fn test_dns_latency(dns_ip: &str) -> Result<u64> {
    let timeout = Duration::from_secs(3);

    // socket UDP
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(timeout))?;
    socket.set_write_timeout(Some(timeout))?;

    // DNS (port 53)
    let addr: SocketAddr = format!("{}:53", dns_ip).parse()?;
    socket.connect(addr)?;

    // Query DNS minimal for "." (root)
    let query: [u8; 17] = [
        0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x01,
    ];
    // idk why the fuck this works

    // time stamp
    let start = Instant::now();
    socket.send(&query)?;

    // wait return
    let mut buf = [0u8; 512];
    socket.recv(&mut buf)?;

    let elapsed = start.elapsed();
    Ok(elapsed.as_millis() as u64)
}

/// tests the latency of multiple DNS servers concurrently
/// uses tokio's spawn_blocking to run the blocking test_dns_latency in parallel
/// WIP
///
// pub async fn test_all_dns(dns_ips: Vec<String>) -> Vec<(String, Result<u64>)> {
//     use tokio::task;
//     let mut handles = Vec::new();
//     for ip in dns_ips {
//         let ip_clone = ip.clone();
//         let handle = task::spawn_blocking(move || {
//             test_dns_latency(&ip_clone)
//         });
//         handles.push((ip, handle));
//     }
//     let mut results = Vec::new();
//     for (ip, handle) in handles {
//         let result = handle.await
//             .map_err(|e| anyhow::anyhow!("Task failed: {}", e))
//             .and_then(|r| r);
//         results.push((ip, result));
//     }
//     results
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // require net
    fn test_cloudflare_latency() {
        let result = test_dns_latency("1.1.1.1");
        assert!(result.is_ok());
        let ms = result.unwrap();
        println!("Cloudflare latency: {}ms", ms);
        assert!(ms < 1000); // should respond in less than 1 second
    }
}
