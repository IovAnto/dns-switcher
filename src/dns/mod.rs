//DNS management module

pub mod manager;
pub mod providers;
pub mod speed;

pub use manager::DnsManager;
pub use providers::{DnsProvider, DEFAULT_PROVIDERS};
pub use speed::test_dns_latency;
