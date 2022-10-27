use lazy_static::lazy_static;
use semver::Version;
use serde::{Deserialize, Serialize};

use versions::serde_helpers::semver_serde;

lazy_static! {
    static ref DEFAULT_KUBERNETES_VERSION: Version = Version::parse("0.0.1").unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) struct Settings {
    #[serde(with = "semver_serde")]
    pub kubernetes_version: Version,
    #[serde(default = "deny_on_deprecation_default")]
    pub deny_on_deprecation: bool,
}

fn deny_on_deprecation_default() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            kubernetes_version: DEFAULT_KUBERNETES_VERSION.clone(),
            deny_on_deprecation: true,
        }
    }
}

impl kubewarden::settings::Validatable for Settings {
    fn validate(&self) -> Result<(), String> {
        if self.kubernetes_version == *DEFAULT_KUBERNETES_VERSION {
            Err("Please provide a kubernetes version".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::settings::Validatable;

    #[test]
    fn validate_settings() {
        let settings = Settings {
            kubernetes_version: Version::parse("1.25.0").unwrap(),
            deny_on_deprecation: true,
        };

        assert!(settings.validate().is_ok());

        let settings = Settings {
            kubernetes_version: Version::parse("0.0.1").unwrap(),
            deny_on_deprecation: true,
        };

        assert!(settings.validate().is_err());
    }
}
