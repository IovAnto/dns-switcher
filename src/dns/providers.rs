#[derive(Debug, Clone)]
pub struct DnsProvider {
    pub id: &'static str,
    pub name: &'static str,
    pub primary: &'static str,
    pub secondary: &'static str,
    pub is_custom: bool,
}

impl DnsProvider {
    /// Create custom DNS provider
    pub fn custom(name: String, primary: String, secondary: String) -> Self {
        // Leak strings to get 'static lifetime - acceptable for custom providers
        let id: &'static str =
            Box::leak(format!("custom_{}", name.to_lowercase().replace(' ', "_")).into_boxed_str());
        let name: &'static str = Box::leak(name.into_boxed_str());
        let primary: &'static str = Box::leak(primary.into_boxed_str());
        let secondary: &'static str = Box::leak(secondary.into_boxed_str());

        Self {
            id,
            name,
            primary,
            secondary,
            is_custom: true,
        }
    }

    /// Return DNS IPs formatted for nmcli
    pub fn dns_string(&self) -> String {
        if self.secondary.is_empty() {
            self.primary.to_string()
        } else {
            format!("{} {}", self.primary, self.secondary)
        }
    }
}

pub const DEFAULT_PROVIDERS: &[DnsProvider] = &[
    DnsProvider {
        id: "google",
        name: "Google",
        primary: "8.8.8.8",
        secondary: "8.8.4.4",
        is_custom: false,
    },
    DnsProvider {
        id: "cloudflare",
        name: "Cloudflare",
        primary: "1.1.1.1",
        secondary: "1.0.0.1",
        is_custom: false,
    },
    DnsProvider {
        id: "opendns",
        name: "OpenDNS",
        primary: "208.67.222.222",
        secondary: "208.67.220.220",
        is_custom: false,
    },
    DnsProvider {
        id: "adguard",
        name: "AdGuard",
        primary: "94.140.14.14",
        secondary: "94.140.15.15",
        is_custom: false,
    },
    DnsProvider {
        id: "quad9",
        name: "Quad9",
        primary: "9.9.9.9",
        secondary: "149.112.112.112",
        is_custom: false,
    },
];

fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }

    for part in parts {
        match part.parse::<u8>() {
            Ok(_) => continue,
            Err(_) => return false,
        }
    }

    true
}

/// Validate DNS input string (one or two IPs)
pub fn validate_dns_input(input: &str) -> Result<(String, String), &'static str> {
    let trimmed = input.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    match parts.len() {
        1 => {
            if is_valid_ip(parts[0]) {
                Ok((parts[0].to_string(), String::new()))
            } else {
                Err("Invalid IP address format")
            }
        }
        2 => {
            if is_valid_ip(parts[0]) && is_valid_ip(parts[1]) {
                Ok((parts[0].to_string(), parts[1].to_string()))
            } else {
                Err("Invalid IP address format")
            }
        }
        _ => Err("Enter one or two IP addresses separated by space"),
    }
}
