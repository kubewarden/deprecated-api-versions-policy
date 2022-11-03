use std::collections::{HashMap, HashSet};
use versions::Versions;

mod cli;
mod policy_metadata;
use policy_metadata::{MetadataLite as PolicyMetadata, Operation, Rule as PolicyMetadataRule};

const K8S_COMPONENT: &str = "k8s";

fn generate_metadata_rules() -> Vec<PolicyMetadataRule> {
    let mut metadata_rules: Vec<PolicyMetadataRule> = vec![];

    let mut relevant_versions: HashMap<String, HashSet<String>> = HashMap::new();

    let versions: Versions = serde_yaml::from_slice(include_bytes!("../../../versions.yaml"))
        .expect("Cannot decode versions.yml");

    for rule in &versions.deprecated_versions {
        if rule.component != K8S_COMPONENT {
            continue;
        }
        let buffer: Vec<&str> = rule.version.splitn(2, '/').collect();
        let (api_group, api_version) = if buffer.len() != 2 {
            ("", buffer[0])
        } else {
            (buffer[0], buffer[1])
        };

        let mut api_versions: HashSet<String> = if relevant_versions.contains_key(api_group) {
            relevant_versions[api_group].clone()
        } else {
            HashSet::new()
        };
        api_versions.insert(api_version.to_string());
        relevant_versions.insert(api_group.to_string(), api_versions);
    }

    let mut api_groups: Vec<String> = relevant_versions.keys().map(|k| k.to_string()).collect();
    api_groups.sort();

    for api_group in &api_groups {
        let mut api_versions: Vec<String> = relevant_versions[api_group].iter().cloned().collect();
        api_versions.sort();

        metadata_rules.push(PolicyMetadataRule {
            api_groups: vec![api_group.to_owned()],
            api_versions,
            resources: vec!["*".to_string()],
            operations: vec![Operation::Create],
        });
    }

    metadata_rules
}

pub fn main() {
    let cli = cli::Cli::new();
    if cli.metadata_path.is_empty() {
        eprintln!("You must provide the path to the metadata.yml file of the policy");
        std::process::exit(1);
    }

    let metadata_raw = std::fs::read(cli.metadata_path).expect("Error reading metadata file");
    let metadata: PolicyMetadata =
        serde_yaml::from_slice(&metadata_raw).expect("Cannot deserialize PolicyMetadata");

    let expected_rules = generate_metadata_rules();

    match &cli.command {
        Some(cli::Commands::Build {}) => {
            println!("{}", serde_yaml::to_string(&expected_rules).unwrap());
        }
        Some(cli::Commands::Check {}) => {
            if metadata.rules != expected_rules {
                eprintln!("Current rules are not correct.");
                std::process::exit(1);
            }
        }
        None => {}
    };
}
