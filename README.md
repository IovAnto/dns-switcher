# dns-switcher

A fast terminal UI to switch DNS providers on Linux, written in Rust.

![demo](demo.gif)

Built on an event-driven loop that only redraws when something changes, so it sits at
roughly 0% CPU while idle.

## Features

- Switch between built-in DNS providers or your own custom entries
- Latency test to see which resolver responds fastest before applying
- Detects the current system DNS on startup
- Add and remove custom providers from the interface
- Reset back to the default configuration
- Layout adapts to the terminal size; keyboard-driven, vim-style navigation

## Requirements

- Linux with `systemd-resolved`
- `iwd` (Wi-Fi handling)
- `polkit` (privilege elevation when applying changes)

## Install

**Arch Linux (AUR):**
```bash
yay -S dns-switcher
```

**Other distributions (install script):**
```bash
curl -sSL https://raw.githubusercontent.com/IovAnto/dns-switcher/main/install.sh | bash
```

**From source (requires Rust):**
```bash
git clone https://github.com/IovAnto/dns-switcher.git
cd dns-switcher
cargo build --release
sudo install -Dm755 target/release/dns-switcher /usr/local/bin/dns-switcher
```

## Usage

```bash
dns-switcher
```

| Key | Action |
|-----|--------|
| `↑` `↓` / `k` `j` | Move selection |
| `Enter` | Apply the selected provider |
| `t` | Latency test |
| `a` | Add a custom provider |
| `d` | Delete the selected provider |
| `r` | Reset DNS to default |
| `h` | Toggle help |
| `q` / `Esc` | Quit |

## Built with

Rust · [ratatui](https://github.com/ratatui/ratatui) · [crossterm](https://github.com/crossterm-rs/crossterm) · [tokio](https://tokio.rs)

## License

MIT
