use regex::Regex;
use spdx::Expression;
use std::borrow::Cow;
use std::sync::LazyLock;

static YEAR_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<year>\s*").unwrap());

pub struct LicenseDefinition {
    license: Expression,
    year: Option<u16>,
    holders: String,
}

impl LicenseDefinition {
    /// Create a new license definition.
    pub fn new(license: Expression, year: Option<u16>, holders: impl Into<String>) -> Self {
        LicenseDefinition {
            license,
            year,
            holders: holders.into(),
        }
    }

    /// Generate the license body with placeholders replaced by year and holders.
    pub fn generate_body(&self) -> Vec<String> {
        self.license
            .requirements()
            .flat_map(|req| req.req.license.id())
            .map(|id| {
                let raw_text = id.text();
                let text_with_holders =
                    raw_text.replace("<copyright holders>", self.holders.as_ref());
                if let Some(year) = self.year {
                    // Replace <year> placeholder with the actual year
                    text_with_holders.replace("<year>", &year.to_string())
                } else {
                    // Remove <year> placeholders if no year is specified
                    let without_year = YEAR_REGEX.replace_all(&text_with_holders, "");
                    without_year.into_owned()
                }
            })
            .collect()
    }
}
