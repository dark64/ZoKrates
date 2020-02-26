use reqwest::blocking::Response;
use reqwest::StatusCode;
use std::path::PathBuf;

use zokrates_core::compile::{ResolvedModule, Resolver};
use zokrates_core::imports::Error as ImportError;

pub struct UrlResolver {}

impl UrlResolver {
    pub fn new() -> Self {
        UrlResolver {}
    }
}

impl Resolver<ImportError> for UrlResolver {
    fn resolve(
        &self,
        _: PathBuf,
        import_location: PathBuf,
    ) -> Result<ResolvedModule, ImportError> {
        let url = import_location.to_str().unwrap();
        let response: Response = reqwest::blocking::get(url)
            .map_err(|_| ImportError::new(format!("{}", "Could not access remote url")))?;

        match response.status() {
            StatusCode::OK => Ok((response.text().unwrap(), import_location.to_owned())),
            _ => Err(ImportError::new(format!(
                "Unable to resolve module ({}): {}",
                url.clone(),
                response.text().unwrap()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn resolve_ok() {
        let url_resolver = UrlResolver::new();
        let res = url_resolver.resolve(
            PathBuf::from("main"),
            PathBuf::from(
                "https://raw.githubusercontent.com/Zokrates/ZoKrates/master/zokrates_cli/examples/add.zok",
            )
        );
        assert!(res.is_ok());
    }

    #[test]
    pub fn resolve_err() {
        let url_resolver = UrlResolver::new();
        let res = url_resolver.resolve(
            PathBuf::from("main"),
            PathBuf::from(
                "https://raw.githubusercontent.com/Zokrates/ZoKrates/master/zokrates_cli/examples/unknown.zok",
            )
        );
        assert!(res.is_err());
    }
}
