#[macro_use]
extern crate lazy_static;
extern crate regex;

use regex::Regex;
use reqwest::blocking::Response;
use reqwest::StatusCode;
use std::path::PathBuf;

use zokrates_core::imports::Error as ImportError;

lazy_static! {
    static ref PROTOCOL_REGEX: Regex = Regex::new(r#"https?://"#).unwrap();
}

pub fn resolve(
    current_location: PathBuf,
    import_location: PathBuf,
) -> Result<(String, PathBuf), ImportError> {
    let url = import_location.to_str().unwrap();
    if PROTOCOL_REGEX.is_match(url.clone()) {
        let response: Response = reqwest::blocking::get(url).unwrap();
        match response.status() {
            StatusCode::OK => Ok((response.text().unwrap(), current_location.to_owned())),
            _ => Err(ImportError::new(format!(
                "Unable to resolve module ({}): {}",
                url.clone(),
                response.text().unwrap()
            ))),
        }
    } else {
        Err(ImportError::new(format!("{}", "Invalid URL; http(s) protocol expected")))
    }
}

pub fn is_external_import(source: &str) -> bool {
    PROTOCOL_REGEX.is_match(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn resolve_ok() {
        let res = resolve(
            PathBuf::new(),
            PathBuf::from(
                "https://raw.githubusercontent.com/Zokrates/ZoKrates/master/zokrates_cli/examples/add.zok",
            )
        );
        assert!(res.is_ok());
    }

    #[test]
    pub fn resolve_err() {
        let res = resolve(
            PathBuf::new(),
            PathBuf::from(
                "https://raw.githubusercontent.com/Zokrates/ZoKrates/master/zokrates_cli/examples/unknown.zok",
            )
        );
        assert!(res.is_err());
    }
}
