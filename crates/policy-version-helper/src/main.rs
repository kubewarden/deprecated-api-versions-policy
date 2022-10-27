use versions::Versions;

mod cli;

const K8S_COMPONENT: &str = "k8s";

fn find_most_recent_k8s_version_mentioned_by_versions_yaml() -> String {
    let versions: Versions = serde_yaml::from_slice(include_bytes!("../../../versions.yaml"))
        .expect("Cannot decode versions.yml");

    let mut most_recent_k8s_version =
        semver::Version::parse("0.0.1").expect("Cannot initialize most_recent_k8s_version");

    for rule in &versions.deprecated_versions {
        if rule.component != K8S_COMPONENT {
            continue;
        }
        if let Some(deprecated_in) = &rule.deprecated_in {
            if deprecated_in > &most_recent_k8s_version {
                most_recent_k8s_version = deprecated_in.to_owned();
            }
        }
        if let Some(removed_in) = &rule.removed_in {
            if removed_in > &most_recent_k8s_version {
                most_recent_k8s_version = removed_in.to_owned();
            }
        }
    }

    most_recent_k8s_version.to_string()
}

fn find_policy_version(manifest_path: &str) -> semver::Version {
    let manifest = cargo_toml::Manifest::from_path(manifest_path)
        .expect("cannot read Cargo.toml of the policy");
    let version_str = manifest.package().version();
    semver::Version::parse(version_str).expect("cannot parse version")
}

pub fn main() {
    let cli = cli::Cli::new();
    if cli.manifest_path.is_empty() {
        eprintln!("You must provide the path to the Cargo.toml file of the policy");
        std::process::exit(1);
    }

    let most_recent_k8s_version = find_most_recent_k8s_version_mentioned_by_versions_yaml();
    let current_policy_version = find_policy_version(&cli.manifest_path);

    let build_version =
        semver::BuildMetadata::new(format!("k8sv{}", most_recent_k8s_version).as_str())
            .expect("Cannot create new build metadata");
    let expected_policy_version = semver::Version {
        build: build_version,
        ..current_policy_version.clone()
    };

    match &cli.command {
        Some(cli::Commands::Build {}) => {
            println!("{}", expected_policy_version);
        }
        Some(cli::Commands::Check {}) => {
            if expected_policy_version == current_policy_version {
                println!("Policy version is correct");
            } else {
                eprintln!(
                    "Policy version should be {} instead of {}",
                    expected_policy_version, current_policy_version
                );
                std::process::exit(1);
            }
        }
        None => {}
    };
}
