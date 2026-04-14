use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;

use crate::config::Config;
use crate::dns::providers::validate_dns_input;
use crate::dns::{test_dns_latency, DnsManager, DnsProvider, DEFAULT_PROVIDERS};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    // collect display name.
    AddingCustomName,
    // collect IP(s).
    AddingCustomIp,
}

pub struct App {
    pub running: bool,
    pub mode: AppMode,
    pub providers: Vec<DnsProvider>,
    pub selected_index: usize,
    // Current active DNS primary IP (if detected).
    pub active_dns: Option<String>,
    // Measured latency per provider id.
    pub latencies: HashMap<&'static str, u64>,
    // Shared input buffer used by modal input modes.
    pub input_buffer: String,
    // Temporary storage between name and IP input steps.
    pub temp_custom_name: String,
    pub status_message: Option<String>,
    pub is_error: bool,
    pub message_time: Option<Instant>,
    pub is_loading: bool,
    pub help_visible: bool,
    pub help_scroll: u16,
    pub show_help_footer: bool,
    config: Config,
    dns_manager: DnsManager,
}

impl App {
    pub fn new(show_help_footer: bool) -> Result<Self> {
        let config = Config::load().unwrap_or_default();

        // Start with built-ins, then append persisted custom providers.
        let mut providers: Vec<DnsProvider> = DEFAULT_PROVIDERS.to_vec();
        providers.extend(config.get_custom_providers());

        let dns_manager = DnsManager::new();
        let active_dns = dns_manager.get_current_dns().ok().flatten();

        Ok(Self {
            running: true,
            mode: AppMode::Normal,
            providers,
            selected_index: 0,
            active_dns,
            latencies: HashMap::new(),
            input_buffer: String::new(),
            temp_custom_name: String::new(),
            status_message: None,
            is_error: false,
            message_time: None,
            is_loading: false,
            help_visible: false,
            help_scroll: 0,
            show_help_footer,
            config,
            dns_manager,
        })
    }

    pub fn next(&mut self) {
        if self.providers.is_empty() {
            return;
        }

        // Wrap around at the end.
        if self.selected_index >= self.providers.len() - 1 {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.providers.is_empty() {
            return;
        }

        // Wrap around at the beginning.
        if self.selected_index == 0 {
            self.selected_index = self.providers.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    pub fn close_help(&mut self) {
        self.help_visible = false;
    }

    pub fn selected_provider(&self) -> Option<&DnsProvider> {
        self.providers.get(self.selected_index)
    }

    pub fn apply_selected_dns(&mut self) {
        let Some(provider) = self.selected_provider().cloned() else {
            return;
        };

        self.is_loading = true;
        self.status_message = None;

        let dns_string = provider.dns_string();

        match self.dns_manager.set_dns(&dns_string) {
            Ok(()) => {
                self.active_dns = Some(provider.primary.to_string());
                self.show_message(format!("DNS changed to {}", provider.name), false);
            }
            Err(e) => {
                self.show_message(format!("Failed to change DNS: {}", e), true);
            }
        }

        self.is_loading = false;
    }

    pub fn reset_dns(&mut self) {
        self.is_loading = true;

        match self.dns_manager.reset_dns() {
            Ok(()) => {
                self.active_dns = None;
                self.show_message("DNS reset to ISP default".to_string(), false);

                // Re-read actual state from system after reset.
                if let Ok(Some(ip)) = self.dns_manager.get_current_dns() {
                    self.active_dns = Some(ip);
                }
            }
            Err(e) => {
                self.show_message(format!("Failed to reset DNS: {}", e), true);
            }
        }

        self.is_loading = false;
    }

    pub fn refresh_active_dns(&mut self) {
        if let Ok(dns) = self.dns_manager.get_current_dns() {
            self.active_dns = dns;
        }
    }

    pub fn test_all_latencies(&mut self) {
        self.is_loading = true;
        self.latencies.clear();

        // Best-effort probing: skip providers that fail to respond.
        for provider in &self.providers {
            if let Ok(ms) = test_dns_latency(provider.primary) {
                self.latencies.insert(provider.id, ms);
            }
        }

        self.is_loading = false;
        self.show_message("Speed test completed".to_string(), false);
    }

    pub fn start_add_custom(&mut self) {
        self.mode = AppMode::AddingCustomName;
        self.input_buffer.clear();
        self.temp_custom_name.clear();
    }

    pub fn handle_input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn handle_input_backspace(&mut self) {
        self.input_buffer.pop();
    }

    pub fn confirm_input(&mut self) {
        match self.mode {
            AppMode::AddingCustomName => {
                if self.input_buffer.trim().is_empty() {
                    self.show_message("Name cannot be empty".to_string(), true);
                    return;
                }

                // Move to IP entry step.
                self.temp_custom_name = self.input_buffer.trim().to_string();
                self.input_buffer.clear();
                self.mode = AppMode::AddingCustomIp;
            }
            AppMode::AddingCustomIp => match validate_dns_input(&self.input_buffer) {
                Ok((primary, secondary)) => {
                    // Add to in-memory list first for immediate UI feedback.
                    let provider = DnsProvider::custom(
                        self.temp_custom_name.clone(),
                        primary.clone(),
                        secondary.clone(),
                    );
                    self.providers.push(provider);

                    // Persist in config for next app launch.
                    if let Err(e) = self.config.add_custom_provider(
                        self.temp_custom_name.clone(),
                        primary,
                        secondary,
                    ) {
                        self.show_message(format!("Failed to save: {}", e), true);
                    } else {
                        self.show_message(
                            format!("Added custom DNS: {}", self.temp_custom_name),
                            false,
                        );
                    }

                    self.cancel_input();
                }
                Err(msg) => {
                    self.show_message(msg.to_string(), true);
                }
            },
            AppMode::Normal => {}
        }
    }

    pub fn cancel_input(&mut self) {
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.temp_custom_name.clear();
    }

    pub fn delete_selected(&mut self) {
        let Some(provider) = self.selected_provider() else {
            return;
        };

        // Guard built-in providers from deletion.
        if !provider.is_custom {
            self.show_message("Cannot delete built-in providers".to_string(), true);
            return;
        }

        let name = provider.name.to_string();

        match self.config.remove_custom_provider(&name) {
            Ok(true) => {
                self.providers.remove(self.selected_index);

                // Keep selection index valid after removal.
                if self.selected_index >= self.providers.len() && self.selected_index > 0 {
                    self.selected_index -= 1;
                }

                self.show_message(format!("Deleted: {}", name), false);
            }
            Ok(false) => {
                self.show_message("Provider not found in config".to_string(), true);
            }
            Err(e) => {
                self.show_message(format!("Failed to delete: {}", e), true);
            }
        }
    }

    fn show_message(&mut self, msg: String, is_error: bool) {
        self.status_message = Some(msg);
        self.is_error = is_error;
        self.message_time = Some(Instant::now());
    }

    pub fn dismiss_message(&mut self) {
        self.status_message = None;
        self.is_error = false;
        self.message_time = None;
    }

    pub fn check_message_timeout(&mut self) {
        if let Some(time) = self.message_time {
            // Auto-hide status messages after 3 seconds.
            if time.elapsed().as_secs() >= 3 {
                self.dismiss_message();
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(true).expect("Failed to initialize app")
    }
}
