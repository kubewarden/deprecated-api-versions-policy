extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::request::GroupVersionKind;

use anyhow::Result;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;

pub mod serde_helpers;

mod deprecations;
use deprecations::{DeprecationRule, DeprecationRules};

lazy_static! {
    pub static ref DEPRECATION_CHECKER: DeprecationChecker = {
        DeprecationChecker::from_yaml(include_bytes!("../../../versions.yaml")).expect(
            "Cannot deserialize the embedded versions. Check the 'versions.yaml' file for errors",
        )
    };
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Versions {
    pub deprecated_versions: DeprecationRules,
}

pub struct DeprecationChecker {
    deprecated_versions_map: HashMap<String, DeprecationRules>,
}

impl DeprecationChecker {
    pub fn from_yaml(data: &[u8]) -> Result<Self, serde_yaml::Error> {
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

    pub fn check(
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
