use anyhow::{bail, Context, Result};
use std::process::{Command, Output};

/// DNS manager using NetworkManager (nmcli)
pub struct DnsManager {
    use_pkexec: bool,
}

impl DnsManager {
    pub fn new() -> Self {
        Self { use_pkexec: true }
    }

    /// Get currently configured DNS
    pub fn get_current_dns(&self) -> Result<Option<String>> {
        let output = Command::new("nmcli")
            .args(["-f", "IP4.DNS", "dev", "show"])
            .output()
            .context("Failed to execute nmcli")?;

        if !output.status.success() {
            bail!("nmcli failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.starts_with("IP4.DNS") {
                if let Some(ip) = line.split_whitespace().last() {
                    return Ok(Some(ip.to_string()));
                }
            }
        }

        Ok(None)
    }

    fn get_active_connection(&self) -> Result<String> {
        let output = Command::new("nmcli")
            .args(["-t", "-f", "NAME", "connection", "show", "--active"])
            .output()
            .context("Failed to get active connection")?;

        if !output.status.success() {
            bail!("Failed to get active connection");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let connection = stdout
            .lines()
            .next()
            .context("No active network connection found")?;

        Ok(connection.to_string())
    }

    /// Set DNS on active connection
    pub fn set_dns(&self, dns_ips: &str) -> Result<()> {
        let connection = self.get_active_connection()?;

        let cmd = format!(
            "nmcli con mod '{}' ipv4.dns '{}' ipv4.ignore-auto-dns yes && nmcli con up '{}'",
            connection, dns_ips, connection
        );

        self.run_privileged(&cmd)?;

        Ok(())
    }

    /// Reset DNS to default (ISP)
    pub fn reset_dns(&self) -> Result<()> {
        let connection = self.get_active_connection()?;

        let cmd = format!(
            "nmcli con mod '{}' ipv4.dns '' ipv4.ignore-auto-dns no && nmcli con up '{}'",
            connection, connection
        );

        self.run_privileged(&cmd)?;

        Ok(())
    }

    /// Run command with elevated privileges (try pkexec, fallback to sudo)
    fn run_privileged(&self, cmd: &str) -> Result<Output> {
        if self.use_pkexec {
            let output = Command::new("pkexec").args(["sh", "-c", cmd]).output();

            match output {
                Ok(out) if out.status.success() => return Ok(out),
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if stderr.contains("dismissed") || stderr.contains("Not authorized") {
                        bail!("Authentication cancelled by user");
                    }
                }
                Err(_) => {}
            }
        }

        // Fallback to sudo
        let output = Command::new("sudo")
            .args(["sh", "-c", cmd])
            .output()
            .context("Failed to execute sudo")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Command failed: {}", stderr);
        }

        Ok(output)
    }

    /// Check if NetworkManager is available
    pub fn is_available() -> bool {
        Command::new("nmcli")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for DnsManager {
    fn default() -> Self {
        Self::new()
    }
}
