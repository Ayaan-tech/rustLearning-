use clap::Parser;
use log::{error,info};
use pingora::prelude::*;
use pingora::server::configuration::Opt;
use pingora_proxy::http_proxy_service;
use pingora_waf::*;
use std::sync::Arc;

#[derive(Debug,Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[args(short, long, default_value = "config/waf_rules.yaml")]
    config:String,
    #[args(short='u',long)]
    upstream_host: Option<String>,
    #[args(short='p',long)]
    upstream_port: Option<u16>,
    #[args(short='1',long,default_value="0.0.0.0")]
    listen_addr:String,
    #[args(short = 'P', long, default_value = "6188")]
    listen_port: u16,
    //Metrics port
    #[args(short='m',long, default_value = "6190")]
    metrics_port: u16,
    #[arg(short = 't', long)]
    testing_mode: bool, 
}
fn main() {
    env_logger::init();
    let args = Args::parse();
    info!("Starting Pingora WAF...");
    info!("Loading configuration from: {}", args.config);
}
