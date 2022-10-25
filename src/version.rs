use anyhow::{anyhow, Result};
use kubewarden::request::GroupVersionKind;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::helpers::option_semver_serde;

lazy_static! {
    pub(crate) static ref DEPRECATION_CHECKER: DeprecationChecker = {
        DeprecationChecker::from_yaml(include_bytes!("../versions.yaml")).expect(
            "Cannot deserialize the embedded versions. Check the 'versions.yaml' file for errors",
        )
    };
}

type DeprecationRules = Vec<DeprecationRule>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct DeprecationRule {
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
    fn includes(&self, kubernetes_version: &semver::Version) -> bool {
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

    pub(crate) fn is_only_deprecated(&self, kubernetes_version: &semver::Version) -> Result<bool> {
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
                "It has been deprecated starting from {}.",
                deprecated_in
            ));
        }

        if let Some(removed_in) = &self.removed_in {
            msgs.push(format!("It has been removed starting from {}.", removed_in));
        }
        msgs.push(format!("It has been replaced by {}.", self.replacement_api));

        write!(f, "{}", msgs.join(" "))
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Versions {
    deprecated_versions: DeprecationRules,
}

pub(crate) struct DeprecationChecker {
    deprecated_versions_map: HashMap<String, DeprecationRules>,
}

impl DeprecationChecker {
    pub(crate) fn from_yaml(data: &[u8]) -> Result<Self, serde_yaml::Error> {
        let versions: Versions = serde_yaml::from_slice(data)?;
        let mut deprecated_versions_map: HashMap<String, DeprecationRules> = HashMap::new();

        for deprecation in &versions.deprecated_versions {
            let deprecations = if deprecated_versions_map.contains_key(&deprecation.version) {
                let mut deprecations = deprecated_versions_map[&deprecation.version].clone();
                deprecations.push(deprecation.to_owned());
                deprecations
            } else {
                vec![deprecation.to_owned()]
            };
            deprecated_versions_map.insert(deprecation.version.clone(), deprecations);
        }

        Ok(DeprecationChecker {
            deprecated_versions_map,
        })
    }

    pub(crate) fn check(
        &self,
        obj: &GroupVersionKind,
        kubernetes_version: &semver::Version,
    ) -> Option<DeprecationRule> {
        if let Some(deprecations) = self
            .deprecated_versions_map
            .get(format!("{}/{}", obj.group, obj.version).as_str())
        {
            deprecations
                .iter()
                .find(move |&deprecation| {
                    deprecation.kind == obj.kind && deprecation.includes(kubernetes_version)
                })
                .cloned()
        } else {
            None
        }
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

    #[test]
    fn check_deprecations() {
        let yaml_data = r#"
deprecated-versions:
    - version: extensions/v1beta1
      kind: ReplicaSet
      deprecated-in: ""
      removed-in: v1.16.0
      replacement-api: apps/v1
      component: k8s
    - version: extensions/v1beta1
      kind: PodSecurityPolicy
      deprecated-in: v1.10.0
      removed-in: v1.16.0
      replacement-api: policy/v1beta1
      component: k8s
"#;

        let deprecated_versions =
            DeprecationChecker::from_yaml(yaml_data.as_bytes()).expect("Cannot parse yaml");

        let obj = GroupVersionKind {
            group: "extensions".to_string(),
            version: "v1beta1".to_string(),
            kind: "ReplicaSet".to_string(),
        };

        let kubernetes_version = Version::parse("1.24.0").expect("Cannot parse version");
        let deprecation = deprecated_versions.check(&obj, &kubernetes_version);
        assert!(deprecation.is_some());

        let kubernetes_version = Version::parse("1.14.0").expect("Cannot parse version");
        let deprecation = deprecated_versions.check(&obj, &kubernetes_version);
        assert!(deprecation.is_none());
    }
}
