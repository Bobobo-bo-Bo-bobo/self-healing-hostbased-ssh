use crate::constants;

pub fn show_usage() {
    global::usage::show_version();
    println!(
        "Usage: {} [-c <cfg>|--config=<cfg>] [-h|--help] [-q|--quiet] [-C|--check] [-D|--debug] [-V|--version]

    -c <cfg>        Read configuration from file <cfg>
    --config=<cfg>  Default: {}

    -h              Shows this text
    --help

    -q              Quiet operation.
    --quiet         Only warning and critical messages will be logged

    -C              Check configuration file and exit
    --check

    -D              Enable debug log
    --debug

    -V              Show version information
    --version

",
        env!("CARGO_BIN_NAME"),
        constants::DEFAULT_CONFIG_FILE
    );
}
