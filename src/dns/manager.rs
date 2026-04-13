use anyhow::{bail, Context, Result};
use std::process::{Command, Output};

/// DNS manager using iwd + systemd-resolved (resolvectl)
pub struct DnsManager {
    use_pkexec: bool,
}

impl DnsManager {
    pub fn new() -> Self {
        Self { use_pkexec: true }
    }

    /// Get currently configured DNS for the active interface.
    pub fn get_current_dns(&self) -> Result<Option<String>> {
        let iface = self.get_active_interface()?;

        let output = Command::new("resolvectl")
            .args(["dns", &iface])
            .output()
            .context("Failed to execute resolvectl")?;

        if !output.status.success() {
            bail!(
                "resolvectl failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let dns_values = stdout.split(':').nth(1).map(str::trim).unwrap_or_default();

        if dns_values.is_empty() {
            return Ok(None);
        }

        let first_ip = dns_values
            .split_whitespace()
            .next()
            .context("Unable to parse DNS from resolvectl output")?;

        Ok(Some(first_ip.to_string()))
    }

    fn get_active_interface(&self) -> Result<String> {
        Self::detect_interface_from_ip_route()
            .or_else(|_| Self::detect_interface_from_iw())
            .or_else(|_| Self::detect_interface_from_iwctl())
            .context("No active network interface found")
    }

    fn detect_interface_from_iwctl() -> Result<String> {
        let output = Command::new("iwctl")
            .args(["station", "list"])
            .output()
            .context("Failed to execute iwctl")?;

        if !output.status.success() {
            bail!("iwctl failed");
        }

        let stdout = Self::strip_ansi(&String::from_utf8_lossy(&output.stdout));
        for line in stdout.lines().skip(1) {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("---") {
                continue;
            }

            if let Some(iface) = trimmed.split_whitespace().next() {
                return Ok(iface.to_string());
            }
        }

        bail!("No interface found from iwctl")
    }

    fn strip_ansi(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut in_escape = false;

        for ch in input.chars() {
            if in_escape {
                if ch == 'm' || ch == 'K' {
                    in_escape = false;
                }
                continue;
            }

            if ch == '\u{1b}' {
                in_escape = true;
                continue;
            }

            out.push(ch);
        }

        out
    }

    fn detect_interface_from_iw() -> Result<String> {
        let output = Command::new("iw")
            .arg("dev")
            .output()
            .context("Failed to execute iw")?;

        if !output.status.success() {
            bail!("iw dev failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if let Some(iface) = trimmed.strip_prefix("Interface ") {
                return Ok(iface.trim().to_string());
            }
        }

        bail!("No interface found from iw dev")
    }

    fn detect_interface_from_ip_route() -> Result<String> {
        let output = Command::new("ip")
            .args(["route", "show", "default"])
            .output()
            .context("Failed to execute ip route")?;

        if !output.status.success() {
            bail!("ip route failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            for window in tokens.windows(2) {
                if window[0] == "dev" {
                    return Ok(window[1].to_string());
                }
            }
        }

        bail!("No default route interface found")
    }

    /// Set DNS on active interface.
    pub fn set_dns(&self, dns_ips: &str) -> Result<()> {
        let iface = self.get_active_interface()?;
        let mut args = vec!["dns".to_string(), iface];
        args.extend(dns_ips.split_whitespace().map(ToString::to_string));
        self.run_resolvectl(&args)?;
        Ok(())
    }

    /// Reset DNS to system/default values.
    pub fn reset_dns(&self) -> Result<()> {
        let iface = self.get_active_interface()?;
        self.run_resolvectl(&["revert".to_string(), iface])?;
        Ok(())
    }

    fn run_resolvectl(&self, args: &[String]) -> Result<Output> {
        let plain = Command::new("resolvectl")
            .args(args)
            .output()
            .context("Failed to execute resolvectl")?;

        if plain.status.success() {
            return Ok(plain);
        }

        if Self::is_running_as_root() {
            bail!("resolvectl failed: {}", Self::output_message(&plain));
        }

        let command_for_escalation = Self::build_resolvectl_command(args);
        self.run_privileged(&command_for_escalation)
            .with_context(|| format!("resolvectl failed: {}", Self::output_message(&plain)))
    }

    /// Run command with elevated privileges (try pkexec, fallback to sudo).
    fn run_privileged(&self, cmd: &str) -> Result<Output> {
        if Self::is_running_as_root() {
            let output = Command::new("sh")
                .args(["-c", cmd])
                .output()
                .context("Failed to execute command as root")?;

            if !output.status.success() {
                bail!("Command failed: {}", Self::output_message(&output));
            }

            return Ok(output);
        }

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

        let output = Command::new("sudo")
            .args(["-n", "sh", "-c", cmd])
            .output()
            .context("Failed to execute sudo")?;

        if output.status.success() {
            return Ok(output);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("a terminal is required") || stderr.contains("a password is required") {
            bail!(
                "Privilege escalation failed. Run `sudo dns-switcher` from a terminal or configure pkexec/polkit."
            );
        }

        let output = Command::new("sudo")
            .args(["sh", "-c", cmd])
            .output()
            .context("Failed to execute sudo")?;

        if !output.status.success() {
            bail!("Command failed: {}", Self::output_message(&output));
        }

        Ok(output)
    }

    fn build_resolvectl_command(args: &[String]) -> String {
        let mut cmd = String::from("resolvectl");
        for arg in args {
            cmd.push(' ');
            cmd.push_str(&Self::shell_escape(arg));
        }
        cmd
    }

    fn shell_escape(s: &str) -> String {
        if s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._:-/".contains(c))
        {
            return s.to_string();
        }

        let escaped = s.replace("'", "'\\''");
        format!("'{}'", escaped)
    }

    fn output_message(output: &Output) -> String {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if !stderr.is_empty() && !stdout.is_empty() {
            format!("{} | {}", stderr, stdout)
        } else if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("exit status {}", output.status)
        }
    }

    fn is_running_as_root() -> bool {
        Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|uid| uid.trim() == "0")
            .unwrap_or(false)
    }

    /// Check if required tooling is available.
    pub fn is_available() -> bool {
        let has_iwd_tooling = Command::new("iwctl")
            .arg("--help")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            || Command::new("iw")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

        let has_resolved = Command::new("resolvectl")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        has_iwd_tooling && has_resolved
    }

    pub fn availability_hint() -> &'static str {
        "This application requires systemd-resolved (resolvectl) and iwd tooling (iwctl or iw)."
    }
}

impl Default for DnsManager {
    fn default() -> Self {
        Self::new()
    }
}
