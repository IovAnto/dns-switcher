# dns-switcher

dns-switcher is a lightweight terminal user interface (TUI) for Arch Linux and other Linux distributions, designed to facilitate rapid switching between different DNS providers. It simplifies the process of changing system DNS settings by providing a clean, responsive interface for selecting both built-in and custom DNS servers.

The application is built with Rust and features a high-performance, asynchronous architecture.

## Features

- Provider Management: Quick selection from a list of popular DNS providers (Cloudflare, Google, Quad9, etc.).
- Custom DNS: Ability to add and persist your own custom DNS server configurations.
- Latency Testing: Real-time speed testing to identify the most responsive DNS provider for your current location.
- Automatic Detection: Identifies and displays the currently active system DNS.
- Adaptive Interface: Responsive design that adjusts its layout for different terminal sizes and heights.
- Resource Efficient: Optimized binary with a small memory footprint.
- Reactive Rendering: Utilizes an event-driven loop that only redraws the UI when necessary, resulting in nearly 0% CPU usage when idle.
- Stealth Mode: Optional `--no-help` flag to hide the footer help menu for a more minimalist experience.

## Screenshots
### Normal mode
![Normal mode](media/img/normal.png)

---
### No help mode
![No help mode](media/img/no_help.png)

---
## Technical Improvements

Starting from version 0.2.1, dns-switcher has been migrated to an asynchronous event loop using Tokio. This migration ensures that the application remains completely idle while waiting for user input or background tasks, drastically reducing its impact on system resources compared to traditional polling-based TUI applications.

## Requirements

- **Linux**: Specifically designed for Linux-based systems.
- **systemd-resolved**: Used to manage and apply DNS settings.
- **iwd**: Required for WiFi network management and DNS assignment.
- **polkit**: Required for non-root execution (handles permission elevation).

## Installation

### AUR (Arch Linux)
The recommended installation method for Arch Linux users:
```bash
yay -S dns-switcher
```

### Installation via Script (Recommended for other distros)
Install the latest pre-compiled binary from GitHub Releases:
```bash
curl -sSL https://raw.githubusercontent.com/IovAnto/dns-switcher/main/install.sh | bash
```

### From Source
If you have the Rust toolchain installed:
```bash
git clone https://github.com/IovAnto/dns-switcher
cd dns-switcher
cargo build --release
sudo install -m755 target/release/dns-switcher /usr/local/bin/
```

## Usage

Launch the application with root privileges if your system requires them to modify DNS settings:
```bash
dns-switcher
```

### Options
- `--no-help`: Starts the application without the help footer, providing more vertical space for the provider list.

### Keybindings
- Arrows / j, k: Navigate the provider list.
- Enter: Apply the selected DNS settings.
- t: Run a latency test for all providers.
- a: Add a custom DNS provider.
- d: Delete a custom provider.
- r: Reset to system/ISP default DNS.
- h: Toggle the help menu.
- q / Esc: Exit the application.

## License

This project is licensed under the MIT License. See the LICENSE file for more information.
