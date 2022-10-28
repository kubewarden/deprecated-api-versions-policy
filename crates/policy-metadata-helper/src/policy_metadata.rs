use serde::{Deserialize, Serialize};

// This is a stripped down copy of `policy_evaluator::policy_metadata`
// We could introduce policy_evaluator as a dependency, but that would
// cause the whole wapc environment to be built for this tiny cli program :/

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetadataLite {
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Operation {
    #[serde(rename = "CREATE")]
    Create,
    #[serde(rename = "UPDATE")]
    Update,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "CONNECT")]
    Connect,
    #[serde(rename = "*")]
    All,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub api_groups: Vec<String>,
    pub api_versions: Vec<String>,
    pub resources: Vec<String>,
    pub operations: Vec<Operation>,
}
