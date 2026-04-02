# DNS Switcher

A terminal user interface (TUI) application for real-time DNS switching on Linux systems using NetworkManager.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## Features

- **Quick DNS Switching**: Switch between popular DNS providers with a single keystroke
- **Pre-configured Providers**: Google, Cloudflare, OpenDNS, AdGuard, and Quad9
- **Custom DNS Servers**: Add and manage your own DNS servers
- **Real-time Status**: Displays the currently active DNS server
- **Speed Testing**: Test latency to all DNS providers
- **Persistent Configuration**: Custom servers are saved and restored automatically
- **Dual Privilege Escalation**: Supports both `pkexec` (PolicyKit) and `sudo`

## Screenshots
<img width="1154" height="532" alt="image" src="https://github.com/user-attachments/assets/74d9e40c-aef3-43fe-834d-ada554a48a47" />


## Installation

### Quick Install (Binary)

Download and install the latest release:

```bash
curl -fsSL https://github.com/IovAnto/dns-switcher/releases/latest/download/dns-switcher-linux-x86_64.tar.gz | tar -xz
sudo install -Dm755 dns-switcher /usr/local/bin/dns-switcher
```

Or use the install script:

```bash
curl -fsSL https://raw.githubusercontent.com/IovAnto/dns-switcher/main/install.sh | bash
```

### From AUR (Arch Linux)

Stable version:
```bash
yay -S dns-switcher
```

Development version (latest git):
```bash
yay -S dns-switcher-git
```

### Using Cargo

If you have Rust installed:

```bash
cargo install --git https://github.com/IovAnto/dns-switcher.git
```

### From Source

#### Prerequisites

- Rust 1.70 or later
- NetworkManager (`nmcli`)
- PolicyKit (`pkexec`) or `sudo`

#### Build and Install

```bash
git clone https://github.com/IovAnto/dns-switcher.git
cd dns-switcher
cargo build --release
sudo install -Dm755 target/release/dns-switcher /usr/local/bin/dns-switcher
```

### Uninstall

```bash
sudo rm /usr/local/bin/dns-switcher
rm -rf ~/.config/dns-switcher
```

## Usage

```bash
dns-switcher
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Apply selected DNS |
| `t` | Test latency of all DNS servers |
| `a` | Add custom DNS server |
| `d` / `Delete` | Delete custom DNS server |
| `r` | Reset to ISP default DNS |
| `q` / `Esc` | Quit |

### Pre-configured DNS Providers

| Provider | Primary | Secondary |
|----------|---------|-----------|
| Google | 8.8.8.8 | 8.8.4.4 |
| Cloudflare | 1.1.1.1 | 1.0.0.1 |
| OpenDNS | 208.67.222.222 | 208.67.220.220 |
| AdGuard | 94.140.14.14 | 94.140.15.15 |
| Quad9 | 9.9.9.9 | 149.112.112.112 |

### Adding Custom DNS Servers

1. Press `a` to add a custom server
2. Enter a name (e.g., "My DNS")
3. Enter IP address(es) (e.g., "1.2.3.4" or "1.2.3.4 5.6.7.8")
4. Press `Enter` to confirm

Custom servers are saved to `~/.config/dns-switcher/config.json`.

## Configuration

Configuration is stored in `~/.config/dns-switcher/config.json`:

```json
{
  "custom_providers": [
    {
      "name": "My Custom DNS",
      "primary": "1.2.3.4",
      "secondary": "5.6.7.8"
    }
  ]
}
```

## Requirements

- **NetworkManager**: This application uses `nmcli` to manage DNS settings
- **Root privileges**: DNS changes require elevated privileges (via `pkexec` or `sudo`)
- **Active network connection**: Must have an active NetworkManager connection

## Technical Details

- DNS is checked every 5 seconds to detect external changes
- Uses NetworkManager's connection modification to change DNS settings
- Supports both IPv4 DNS servers
- Speed test sends a minimal DNS query to measure response time

## Troubleshooting

### "NetworkManager (nmcli) is not available"

Install NetworkManager:
```bash
# Arch Linux
sudo pacman -S networkmanager

# Debian/Ubuntu
sudo apt install network-manager
```

### "Authentication cancelled by user"

The application requires root privileges to change DNS. Make sure to authenticate when prompted.

### DNS change doesn't persist after reboot

This is expected behavior. The application modifies the current connection settings, which may be reset by DHCP. Consider using NetworkManager's persistent DNS settings for permanent changes.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the DNS Switcher plugin for [Noctalia](https://github.com/noctalia-dev/noctalia)
- Built with [Ratatui](https://github.com/ratatui-org/ratatui) - A Rust library for building terminal user interfaces
