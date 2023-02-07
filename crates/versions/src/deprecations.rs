use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::serde_helpers::option_semver_serde;

pub type DeprecationRules = Vec<DeprecationRule>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DeprecationRule {
    pub version: String,
    pub kind: String,
    #[serde(with = "option_semver_serde")]
    pub deprecated_in: Option<semver::Version>,
    #[serde(default)]
    #[serde(with = "option_semver_serde")]
    pub removed_in: Option<semver::Version>,
    pub replacement_api: String,
    pub component: String,
}

impl DeprecationRule {
    pub fn includes(&self, kubernetes_version: &semver::Version) -> bool {
        if let Some(removed_in) = &self.removed_in {
            if kubernetes_version >= removed_in {
                return true;
            }
        }
        if let Some(deprecated_in) = &self.deprecated_in {
            if kubernetes_version >= deprecated_in {
                return true;
            }
        }

        false
    }

    pub fn is_only_deprecated(&self, kubernetes_version: &semver::Version) -> Result<bool> {
        if let Some(removed_in) = &self.removed_in {
            if kubernetes_version >= removed_in {
                return Ok(false);
            }
        }
        if let Some(deprecated_in) = &self.deprecated_in {
            if kubernetes_version >= deprecated_in {
                return Ok(true);
            }
        }

        Err(anyhow!(
            "The deprecation rule '{}' does not apply to kubernetes version: {}",
            self,
            kubernetes_version
        ))
    }
}

impl fmt::Display for DeprecationRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut msgs = vec![format!("{} {} cannot be used.", self.version, self.kind,)];
        if let Some(deprecated_in) = &self.deprecated_in {
            msgs.push(format!(
                "It has been deprecated starting from {deprecated_in}."
            ));
        }

        if let Some(removed_in) = &self.removed_in {
            msgs.push(format!("It has been removed starting from {removed_in}."));
        }
        msgs.push(format!("It has been replaced by {}.", self.replacement_api));

        write!(f, "{}", msgs.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn deserialize_deprecation() {
        let yaml_data = vec![
            r#"
version: extensions/v1beta1
kind: ReplicaSet
deprecated-in: ""
removed-in: v1.16.0
replacement-api: apps/v1
component: k8s"#,
            r#"
version: extensions/v1beta1
kind: PodSecurityPolicy
deprecated-in: v1.10.0
removed-in: v1.16.0
replacement-api: policy/v1beta1
component: k8s"#,
        ];

        for yaml in &yaml_data {
            let deprecation: Result<DeprecationRule, serde_yaml::Error> =
                serde_yaml::from_str(yaml);
            assert!(deprecation.is_ok());
        }
    }

    #[test]
    fn deprecation_applies_to_kubernetes_release() {
        let deprecation: DeprecationRule = serde_yaml::from_str(
            r#"
version: extensions/v1beta1
kind: PodSecurityPolicy
deprecated-in: v1.10.0
removed-in: v1.16.0
replacement-api: policy/v1beta1
component: k8s"#,
        )
        .expect("cannot deserialize");

        let matching_versions = vec!["1.10.0", "1.12.0", "1.16.0", "1.23.0"];
        for v in &matching_versions {
            let kubernetes_version =
                Version::parse(v).unwrap_or_else(|_| panic!("cannot parse version {}", v));
            assert!(
                deprecation.includes(&kubernetes_version),
                "{} should cause a match",
                v
            );
        }

        let matching_versions = vec!["1.0.0", "1.9.99"];
        for v in &matching_versions {
            let kubernetes_version =
                Version::parse(v).unwrap_or_else(|_| panic!("cannot parse version {}", v));
            assert!(
                !deprecation.includes(&kubernetes_version),
                "{} should not cause a match",
                v
            );
        }
    }
}
