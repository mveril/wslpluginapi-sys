use crate::licence_definition::LicenseDefinition;
use quick_xml::DeError;
use quick_xml::de::from_reader;
use regex::Regex;
use serde::{Deserialize, Serialize};
use spdx::Expression;
use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "package")]
pub struct Package {
    #[serde(rename = "metadata")]
    pub metadata: Metadata,
}

impl Package {
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self, DeError> {
        let package: Package = from_reader(reader)?;
        Ok(package)
    }
}

#[derive(Debug, Clone)]
pub enum LicenceContent {
    Body(LicenceBody),
    URL(String),
}

impl From<LicenceBody> for LicenceContent {
    fn from(body: LicenceBody) -> Self {
        LicenceContent::Body(body)
    }
}

#[derive(Debug, Clone)]
pub enum LicenceBody {
    Generator(LicenseDefinition),
    File(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub id: String,
    pub version: String,
    pub authors: String,
    #[serde(default)]
    pub owners: Option<String>,
    #[serde(default)]
    pub readme: Option<PathBuf>,
    #[serde(default)]
    pub copyright: Option<String>,
    pub description: String,
    #[serde(default)]
    pub release_notes: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub project_url: Option<String>,
    #[serde(default)]
    pub license_url: Option<String>,
    #[serde(default)]
    pub license: Option<License>,
    #[serde(default)]
    pub require_license_acceptance: Option<bool>,
    #[serde(default)]
    pub dependencies: Option<Dependencies>,
}

impl Metadata {
    pub fn get_year(&self) -> Option<u16> {
        let re = Regex::new(r"\d{4}").unwrap();
        self.copyright
            .as_deref()
            .map(|copyright| re.captures(&copyright).map(|year| year[0].parse().unwrap()))
            .flatten()
    }

    pub fn get_holders(&self) -> &str {
        if let Some(owners) = &self.owners {
            owners
        } else {
            &self.authors
        }
    }

    pub fn get_licence_content(&self) -> anyhow::Result<Option<LicenceContent>> {
        let year = self.get_year();
        let holders = self.get_holders();
        Ok(if let Some(license) = &self.license {
            Some(LicenceContent::Body(license.get_body(year, &holders)?))
        } else if let Some(license_url) = &self.license_url {
            Some(LicenceContent::URL(license_url.clone()))
        } else {
            None
        })
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
    #[serde(rename = "@type")]
    pub kind: LicenseType,
    #[serde(rename = "$value")]
    value: String,
}

impl License {
    pub fn get_body(&self, year: Option<u16>, holders: &str) -> anyhow::Result<LicenceBody> {
        let result = match self.kind {
            LicenseType::File => {
                let path = Path::new(&self.value);
                LicenceBody::File(fs::read_to_string(path)?)
            }
            LicenseType::Expression => LicenceBody::Generator(LicenseDefinition::new(
                Expression::parse(&self.value)?,
                year,
                holders,
            )),
        };
        Ok(result)
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
