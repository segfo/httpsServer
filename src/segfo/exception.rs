pub mod Exception{
    #[derive(Debug)]
    pub enum ConfigException{
        ParserException(::serde_json::Error),
        IoException(::std::io::Error),
    }

    impl From<::serde_json::Error> for ConfigException {
        fn from(err: ::serde_json::Error) -> ConfigException {
            ConfigException::ParserException(err)
        }
    }

    impl From<::std::io::Error> for ConfigException {
        fn from(err: ::std::io::Error) -> ConfigException {
            ConfigException::IoException(err)
        }
    }

    impl ::std::fmt::Display for ConfigException {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                ConfigException::ParserException(ref err) => write!(f, "Parse Exception: {}", err),
                ConfigException::IoException(ref err) => write!(f, "IO Exception: {}", err),
            }
        }
    }

    impl ::std::error::Error for ConfigException {
        fn description(&self) -> &str {
            match *self {
                ConfigException::ParserException(ref err) => err.description(),
                ConfigException::IoException(ref err) => err.description(),
            }
        }

        fn cause(&self) -> Option<&::std::error::Error> {
            match *self {
                ConfigException::ParserException(ref err) => Some(err),
                ConfigException::IoException(ref err) => Some(err),
            }
        }
    }
}
