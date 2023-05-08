use std::sync::Arc;
use dashmap::DashMap;
use memmap2::Mmap;
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialOrd, PartialEq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub authors: Option<Vec<String>>,
    pub name: String,
    pub version: Version,
    pub compatibility: Option<Version>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>
}