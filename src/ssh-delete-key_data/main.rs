mod config;
mod constants;
mod mqtt;
mod usage;

use getopts::Options;
use log::{debug, error, info};
use std::{env, process};

fn main() {
    let argv: Vec<String> = env::args().collect();
    let mut options = Options::new();
    let mut log_level = log::LevelFilter::Info;

    options.optflag("C", "check", "Check configuration file and exit");
    options.optflag("D", "debug", "Enable debug output");
    options.optflag("V", "version", "Show version information");
    options.optflag("h", "help", "Show help text");
    options.optopt(
        "c",
        "config",
        "Configuration file",
        constants::DEFAULT_CONFIG_FILE,
    );
    options.optflag("q", "quiet", "Quiet operation");

    let opts = match options.parse(&argv[1..]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: Can't parse command line arguments: {}", e);
            println!();
            usage::show_usage();
            process::exit(1);
        }
    };

    if opts.opt_present("h") {
        usage::show_usage();
        process::exit(0);
    }

    if opts.opt_present("V") {
        usage::show_usage();
        process::exit(0);
    }

    if opts.opt_present("D") {
        log_level = log::LevelFilter::Debug;
    }

    if opts.opt_present("q") {
        log_level = log::LevelFilter::Warn;
    }

    let config_file = match opts.opt_str("c") {
        Some(v) => v,
        None => constants::DEFAULT_CONFIG_FILE.to_string(),
    };

    // XXX: Initialisation of logging should never fail
    global::logging::init(log_level).unwrap();

    let configuration = match config::parse_config_file(&config_file) {
        Ok(v) => v,
        Err(e) => {
            error!(
                "parsing of configuration file {} failed: {}",
                config_file, e
            );
            process::exit(1);
        }
    };

    if opts.opt_present("C") {
        info!("configurtion file {} is valid", config_file);
        process::exit(0);
    }

    debug!("parsed configuration: {:?}", configuration);

    if opts.free.is_empty() {
        error!("Missing list of hosts");
        usage::show_usage();
        process::exit(1);
    }

    for host in opts.free.iter() {
        if let Err(e) = config::validate_hostname(host) {
            error!("{}", e);
            process::exit(1);
        }
    }

    if let Err(e) = mqtt::send(&configuration, opts.free) {
        error!("MQTT operation failed: {}", e);
        process::exit(1);
    }

    process::exit(0);
}
