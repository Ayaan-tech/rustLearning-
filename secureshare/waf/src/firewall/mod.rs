pub mod sql_injection;
pub mod body_inspector;
pub mod bot_detector;
pub mod command_injection;
pub mod ip_filter;
pub mod path_traversal;
pub mod rate_limiter;
pub mod rules;
pub mod sql_injection;
pub mod xss_detector;


pub use sql_injection::*;
pub use body_inspector::*;
pub use bot_detector::*;
pub use command_injection::*;
pub use ip_filter::*;
pub use path_traversal::*;
pub use rate_limiter::*;
pub use rules::*;
pub use sql_injection::*;
pub use xss_detector::*;

use pingora::http::RequestHeader;

#[derive(Debug,Clone)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}
#[derive(Debug,Clone)]
pub struct SecurityViolation {
    pub threat_type: String,
    pub threat_level: String,
    pub description: String,
    pub blocked: bool,
}
pub trait SecurityRule: Send + Sync {
    fn check(&self, request: &RequestHandler,body: Option<&[u8]>) -> Result<(), SecurityViolation>;
    fn name(&self) -> &str;
}