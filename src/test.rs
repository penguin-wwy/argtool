#[cfg(test)]
mod tests {
    use ::OptParser;
    use error::Fail::*;

    #[test]
    fn test_long() {
        let long_args = vec!("--test=20".to_string());
        let mut opts = OptParser::new();
        opts.add_optional_arg("t", "test", "test times", "=times");
        let m = opts.parse_arguments(long_args).unwrap();
        let ret = m.get_val("test").unwrap();
        assert_eq!(ret, "20");
    }

    #[test]
    fn test_short() {
        let short_args = vec!("-t".to_string(), "20".to_string());
        let mut opts = OptParser::new();
        opts.add_optional_arg("t", "test", "test times", "=times");
        let m = opts.parse_arguments(short_args).unwrap();
        let ret = m.get_val("test").unwrap();
        assert_eq!(ret, "20");
    }

    #[test]
    fn test_report_missing() {
        let args = vec!("--test=20".to_string());
        let mut opts = OptParser::new();
        opts.add_optional_arg("t", "test", "test times", "=times");
        opts.add_necessary_flag("i", "index", "include mode", "");
        match opts.parse_arguments(args) {
            Err(MissingArgument(nm)) => {
                println!("missing argument: {}", nm);
            }
            _ => panic!("is not missing argument")
        }
    }

    #[test]
    fn test_report_unknown() {
        let args = vec!("-i".to_string());
        let mut opts = OptParser::new();
        opts.add_optional_arg("t", "test", "test times", "=times");
        match opts.parse_arguments(args) {
            Err(UnknownArgument(nm)) => {
                println!("unknown argument: {}", nm);
            }
            _ => panic!("is not unknown argument")
        }
    }

    #[test]
    fn test_report_duplicated() {
        let args = vec!("-t".to_string(), "20".to_string(), "-t".to_string(), "15".to_string());
        let mut opts = OptParser::new();
        opts.add_optional_arg("t", "test", "test times", "=times");
        match opts.parse_arguments(args) {
            Err(DuplicatedArgument(nm)) => {
                println!("duplicated argument: {}", nm);
            }
            _ => panic!("is not duplicated argument")
        }
    }

    #[test]
    fn test_report_unexpected() {
        let args = vec!("--test=20".to_string());
        let mut opts = OptParser::new();
        opts.add_optional_flag("t", "test", "run test", "");
        match opts.parse_arguments(args) {
            Err(UnexpectedArgument(nm)) => {
                println!("unexpected argument: {}", nm);
            }
            _ => panic!("is not unexpected argument")
        }

        let args = vec!("-t".to_string(), "20".to_string(), "file".to_string());
        let mut opts = OptParser::new();
        opts.choose_strict_style().add_optional_arg("t", "test", "test time", "=times");
        match opts.parse_arguments(args) {
            Err(UnexpectedArgument(nm)) => {
                println!("unexpected argument: {}", nm);
            }
            _ => panic!("is not unexpected argument")
        }

        let args = vec!("-t".to_string());
        let mut opts = OptParser::new();
        opts.add_optional_arg("t", "test", "test times", "=times");
        match opts.parse_arguments(args) {
            Err(UnexpectedArgument(nm)) => {
                println!("unexpected argument: {}", nm);
            }
            _ => panic!("is not missing argument")
        }
    }

    #[test]
    fn test_usage() {
        let mut opts = OptParser::new();
        opts.add_optional_arg("t", "test", "Specifies the test times", "TIMES");
        opts.add_optional_arg("f", "file", "Specifies the input file", "");
        opts.add_optional_arg("", "sdk", "Specifies the sdk path", "sdk_path");
        println!("{}", opts.usage("this software arguments:"));
    }
}