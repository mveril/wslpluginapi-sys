use chrono::prelude::*;
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use spdx::{Expression, text};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "package")]
pub struct Package {
    #[serde(rename = "metadata")]
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "metadata")]
pub struct Metadata {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "version")]
    pub version: String,
    #[serde(rename = "authors")]
    pub authors: String,
    #[serde(rename = "owners")]
    #[serde(default)]
    pub owners: Option<String>,
    #[serde(default)]
    pub copyright: Option<String>,
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "releaseNotes")]
    #[serde(default)]
    pub release_notes: Option<String>,
    #[serde(rename = "tags")]
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(rename = "projectUrl")]
    #[serde(default)]
    pub project_url: Option<String>,
    #[serde(rename = "licenseUrl")]
    #[serde(default)]
    pub license_url: Option<String>,
    #[serde(rename = "license")]
    #[serde(default)]
    pub license: Option<License>,
    #[serde(rename = "requireLicenseAcceptance")]
    #[serde(default)]
    pub require_license_acceptance: Option<bool>,
    #[serde(rename = "dependencies")]
    #[serde(default)]
    pub dependencies: Option<Dependencies>,
}

impl Metadata {
    pub fn get_year(&self) -> Option<i32> {
        let re = Regex::new(r"\d{4}").unwrap();
        self.copyright
            .as_deref()
            .map(|copyright| {
                re.captures(&copyright)
                    .map(|year| year[0].parse::<i32>().unwrap())
            })
            .flatten()
    }
    pub fn get_holders(&self) -> &str {
        if let Some(owners) = &self.owners {
            owners
        } else {
            &self.authors
        }
    }

    pub fn get_licence_texts(&self) -> Vec<String> {
        let year = self.get_year();
        let holders = self.get_holders();
        if let Some(license) = &self.license {
            license.get_text(year, &holders)
        } else if let Some(license_url) = &self.license_url {
            vec![format!("See {}", license_url)]
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum LicenseType {
    Expression,
    File,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    #[serde(rename = "type")]
    pub kind: LicenseType,
    #[serde(rename = "$value")]
    value: String,
}

impl License {
    pub fn expression(&self) -> Option<Expression> {
        if self.kind == LicenseType::Expression {
            Expression::parse(&self.value).ok()
        } else {
            None
        }
    }

    pub fn file(&self) -> Option<String> {
        if self.kind == LicenseType::File {
            Some(self.value.clone())
        } else {
            None
        }
    }

    pub fn get_text(&self, year: Option<i32>, holders: &str) -> Vec<String> {
        match self.kind {
            LicenseType::File => {
                let path = Path::new(&self.value);
                vec![fs::read_to_string(path).expect("Failed to read license file")]
            }
            LicenseType::Expression => {
                let licence_expr =
                    Expression::parse(&self.value).expect("Failed to parse license expression");

                let year_regex = Regex::new(r"<year>\s*").unwrap();

                licence_expr
                    .requirements()
                    .flat_map(|req| req.req.license.id())
                    .map(|id| {
                        let raw_text = id.text();
                        let text_with_holders = raw_text.replace("<copyright holders>", holders);
                        let text = if let Some(year) = year {
                            text_with_holders.replace("<year>", &year.to_string())
                        } else {
                            let without_year = year_regex.replace_all(&raw_text, "");
                            without_year.replace("<copyright holders>", holders)
                        };
                        text
                    })
                    .collect()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "dependencies")]
pub struct Dependencies {
    #[serde(rename = "group")]
    #[serde(default)]
    pub group: Vec<DependencyGroup>,
    #[serde(rename = "dependency")]
    #[serde(default)]
    pub dependency: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "group")]
pub struct DependencyGroup {
    #[serde(rename = "@targetFramework")]
    #[serde(default)]
    pub target_framework: Option<String>,
    #[serde(rename = "dependency")]
    pub dependency: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "dependency")]
pub struct Dependency {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@exclude")]
    #[serde(default)]
    pub exclude: Option<String>,
}
