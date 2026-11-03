use super::{SecurityRule,SecurityViolation,ThreatLevel};
use once_cell::sync::Lazy;
use pingora::http::RequestHeader;
use regex::Regex;
use std::collections::HashSet;

static BAD_BOT_PATTERNS : Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
         // Security scanners and vulnerability tools
        Regex::new(r"(?i)\bsqlmap\b").unwrap(),
        Regex::new(r"(?i)\bnikto\b").unwrap(),
        Regex::new(r"(?i)\bnmap\b").unwrap(),
        Regex::new(r"(?i)\bmasscan\b").unwrap(),
        Regex::new(r"(?i)\bmetasploit\b").unwrap(),
        Regex::new(r"(?i)\bburpsuite\b").unwrap(),
        Regex::new(r"(?i)\bacunetix\b").unwrap(),
        Regex::new(r"(?i)\bnessus\b").unwrap(),
        Regex::new(r"(?i)\bowasp\b.*\bzap\b").unwrap(),
        Regex::new(r"(?i)\bdirbuster\b").unwrap(),
        Regex::new(r"(?i)\bgobuster\b").unwrap(),
        Regex::new(r"(?i)\bwpscan\b").unwrap(),
        Regex::new(r"(?i)\bjoomscan\b").unwrap(),
        Regex::new(r"(?i)\bw3af\b").unwrap(),
        Regex::new(r"(?i)\barachni\b").unwrap(),
        Regex::new(r"(?i)\bskipfish\b").unwrap(),
        // Web scrapers
        Regex::new(r"(?i)\bscrapy\b").unwrap(),
        Regex::new(r"(?i)\bwebharvest\b").unwrap(),
        Regex::new(r"(?i)\bhttrack\b").unwrap(),
        Regex::new(r"(?i)\bwebcopier\b").unwrap(),
        Regex::new(r"(?i)\boffline\s*explorer\b").unwrap(),
        Regex::new(r"(?i)\bteleport\s*pro\b").unwrap(),
        Regex::new(r"(?i)\bwebzip\b").unwrap(),
        //Spam bots
        Regex::new(r"(?i)\bsemrush\b").unwrap(),
        Regex::new(r"(?i)\bahrefs\b").unwrap(),
        Regex::new(r"(?i)\bmj12bot\b").unwrap(),
        Regex::new(r"(?i)\bdotbot\b").unwrap(),
        Regex::new(r"(?i)\bseekport\b").unwrap(),
        Regex::new(r"(?i)\bblexbot\b").unwrap(),
        //Generic bad patterns
        Regex::new(r"(?i)\bcrawler\b.*\bbot\b").unwrap(),
        Regex::new(r"(?i)\bspider\b.*\bbot\b").unwrap(),
        Regex::new(r"(?i)\bscan\b").unwrap(),
        Regex::new(r"(?i)\bharvest\b").unwrap(),
        Regex::new(r"(?i)\bextract\b").unwrap(),
        //Library defaults that are often abused
        Regex::new(r"^python-requests").unwrap(),
        Regex::new(r"^python-urllib").unwrap(),
        Regex::new(r"^Java/").unwrap(),
        Regex::new(r"^libwww-perl").unwrap(),
        Regex::new(r"^Go-http-client").unwrap(),
        Regex::new(r"^curl/").unwrap(),
        Regex::new(r"^wget/").unwrap(),

    ]
})
static GOOD_BOD_PATTERNS : Lazy<HashSet<& 'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("googlebot");
    set.insert("bingbot");
    set.insert("slurp"); // Yahoo
    set.insert("duckduckbot");
    set.insert("baiduspider");
    set.insert("yandexbot");
    set.insert("sogou");
    set.insert("exabot");
    set.insert("facebot"); // Facebook
    set.insert("facebookexternalhit");
    set.insert("ia_archiver"); // Alexa
    // Social media
    set.insert("twitterbot");
    set.insert("linkedinbot");
    set.insert("pinterestbot");
    set.insert("slackbot");
    set.insert("telegrambot");
    set.insert("discordbot");
    set.insert("whatsapp");
    // Monitoring and verification
    set.insert("uptimerobot");
    set.insert("pingdom");
    set.insert("statuscake");
    set.insert("site24x7");
    set.insert("gtmetrix");
    // Feed readers
    set.insert("feedly");
    set.insert("feedburner");
})
//Bot detection result
#[derive(Debug,Clone,PartialEq)]
pub enum BotType {
    GoodBot(String),
    BadBot(String),
    SuspiciousBot(String),
    NotBot,
}
pub struct BotDetector {
    pub enabled: bool,
    pub block_mode: bool,
    pub allow_known_bots: bool,
    custom_bad_patterns: Vec<Regex>,
    custom_good_identifiers: HashSet<String>,
}
impl BotDetector {
    //Create a new bot 
    /// * `enabled` - Whether bot detection is enabled
    /// * `block_mode` - true = block bad bots, false = log only
    /// * `allow_known_bots` - Allow known good bots (Googlebot, Bingbot, etc.)
    pub fn new(enabled:bool, block_mode: bool, allow_known_bots: bool) -> Self {
        Self {
            enabled,
            block_mode,
            allow_known_bots,
            custom_bad_patterns: Vec::new(),
            custom_good_identifiers:HashSet::new(),
        }
    }
    
}