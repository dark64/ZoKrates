#[macro_use]
extern crate lazy_static;
extern crate regex;

pub mod resolvers;

use regex::Regex;
use resolvers::*;
use std::path::PathBuf;
use zokrates_core::compile::{ResolvedModule, Resolver};
use zokrates_core::imports::Error;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r#"^https?://"#).unwrap();
}

pub struct SmartResolver {}
impl SmartResolver {
    pub fn new() -> Self {
        SmartResolver {}
    }
}

impl Resolver<Error> for SmartResolver {
    fn resolve(
        &self,
        current_location: PathBuf,
        import_location: PathBuf,
    ) -> Result<ResolvedModule, Error> {
        if URL_REGEX.is_match(import_location.to_str().unwrap()) {
            url::UrlResolver::new().resolve(current_location, import_location)
        } else {
            fs::FileSystemResolver::new()
                .resolve(current_location, import_location)
                .map_err(|e| e.into())
        }
    }
}
