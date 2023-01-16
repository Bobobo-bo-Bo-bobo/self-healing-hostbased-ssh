mod collect;
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
    options.optopt(
        "d",
        "ssh-directory",
        "Directory containing SSH keys",
        global::constants::DEFAULT_SSH_DIRECTORY,
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

    let mut configuration = match config::parse_config_file(&config_file) {
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
        info!("configuration file {} is valid", config_file);
        process::exit(0);
    }

    configuration.ssh_directory = match opts.opt_str("d") {
        Some(v) => v,
        None => global::constants::DEFAULT_SSH_DIRECTORY.to_string(),
    };

    config::fill_missing_fields(&mut configuration).unwrap();
    if configuration.ssh_keys.files.is_empty() {
        error!("no public ssh keys found; neither provided by the configuration file nor in the directory {}", configuration.ssh_directory);
        process::exit(1);
    }

    debug!("parsed configuration: {:?}", configuration);
    let keys = match collect::read_key_files(&configuration) {
        Ok(v) => v,
        Err(e) => {
            error!("can't read key files: {}", e);
            process::exit(1);
        }
    };
    debug!("parsed key data: {:?}", keys);

    if let Err(e) = mqtt::send(&configuration, &keys) {
        error!("{}", e);
        process::exit(1);
    };

    process::exit(0);
}
