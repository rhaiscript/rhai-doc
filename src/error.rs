use std::convert::From;
use std::fmt;

#[derive(Debug)]
pub enum RhaiDocError {
    Internal(String),
    Icon(String),
}

impl fmt::Display for RhaiDocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RhaiDocError::Internal(message) => write!(f, "{}", message),
            RhaiDocError::Icon(message) => write!(f, "Icon Error: {}", message),
        }
    }
}

macro_rules! impl_error {
    { $ERROR_TYPE:ty } => {
        impl From<$ERROR_TYPE> for RhaiDocError {
            fn from(error: $ERROR_TYPE) -> Self {
                RhaiDocError::Internal(format!("{}", error))
            }
        }
     };
}

impl_error!(handlebars::TemplateError);
impl_error!(glob::PatternError);
impl_error!(std::str::Utf8Error);
impl_error!(std::boxed::Box<rhai::EvalAltResult>);
impl_error!(handlebars::RenderError);
impl_error!(std::io::Error);
impl_error!(toml::de::Error);
impl_error!(std::path::StripPrefixError);
