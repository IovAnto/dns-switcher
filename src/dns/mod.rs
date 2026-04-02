//DNS management module

pub mod providers;
pub mod manager;
pub mod speed;

pub use providers::{DnsProvider, DEFAULT_PROVIDERS};
pub use manager::DnsManager;
pub use speed::test_dns_latency;