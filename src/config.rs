#[derive(Debug, PartialEq)]
pub struct Config {
    pub filename: String,
    pub debug: bool,
}

impl Config {
    pub fn with(args: &[String]) -> Result<Self, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let mut config = Self {
            filename: "".to_string(),
            debug: false,
        };

        for arg in args {
            if arg == "--debug" {
                config.debug = true;
            } else {
                config.filename = arg.clone();
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_valid_arguments() {
        let args = [String::from("program_name"), String::from("filename")].to_vec();

        assert_eq!(
            Config::with(&args).unwrap().filename,
            String::from("filename")
        );
        assert_eq!(Config::with(&args).unwrap().debug, false);
    }

    #[test]
    fn config_not_enough_arguments() {
        let args = [String::from("program_name")].to_vec();

        assert_eq!(Config::with(&args), Err("not enough arguments"));
    }

    #[test]
    fn config_with_debug() {
        let args = [
            String::from("program_name"),
            String::from("filename"),
            String::from("--debug"),
        ]
        .to_vec();

        assert_eq!(
            Config::with(&args).unwrap().filename,
            String::from("filename")
        );
        assert_eq!(Config::with(&args).unwrap().debug, true);
    }

    #[test]
    fn config_with_debug_first() {
        let args = [
            String::from("program_name"),
            String::from("--debug"),
            String::from("filename"),
        ]
        .to_vec();

        assert_eq!(
            Config::with(&args).unwrap().filename,
            String::from("filename")
        );
        assert_eq!(Config::with(&args).unwrap().debug, true);
    }
}
