#[cfg(test)]
mod config_util_tests {
    use super::super::config_util::*;
    const ARGS: [&str; 9] = [
        "run",
        "--pv-api-key", "",
        "--pv-model-path", "model.rhn",
        "--mic-index", "0",
        "--vision-type", "none"];
    #[test]
    fn cli_parse_command() {
        let cli = Cli::new(ARGS.iter().map(|s| s.to_string()).collect::<Vec<String>>());
        assert_eq!(cli.parse_command().unwrap(), Command::Run);
    }

    #[test]
    fn cli_parse_config() {
        let cli = Cli::new(ARGS.iter().map(|s| s.to_string()).collect::<Vec<String>>());
        assert_eq!(cli.parse_config().unwrap(), Config::new());
    }
}