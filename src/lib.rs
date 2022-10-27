use guest::prelude::*;
use kubewarden_policy_sdk::wapc_guest as guest;

extern crate kubewarden_policy_sdk as kubewarden;
use kubewarden::{protocol_version_guest, request::ValidationRequest, validate_settings};

mod settings;
use settings::Settings;

use versions::DEPRECATION_CHECKER;

#[no_mangle]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
    register_function("protocol_version", protocol_version_guest);
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;
    let obj = validation_request.request.kind;
    let kubernetes_version = validation_request.settings.kubernetes_version;

    match DEPRECATION_CHECKER.check(&obj, &kubernetes_version) {
        Some(deprecation_rule) => {
            if !validation_request.settings.deny_on_deprecation
                && deprecation_rule.is_only_deprecated(&kubernetes_version)?
            {
                return kubewarden::accept_request();
            }
            kubewarden::reject_request(Some(deprecation_rule.to_string()), None, None, None)
        }
        None => kubewarden::accept_request(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kubewarden_policy_sdk::test::Testcase;

    #[test]
    fn eval_extensions_v1beta1_ingress() -> Result<(), ()> {
        // This is the deprecation rule:
        //
        //    version: extensions/v1beta1
        //    kind: Ingress
        //    deprecated-in: v1.14.0
        //    removed-in: v1.22.0
        //    replacement-api: networking.k8s.io/v1
        //    component: k8s

        let request_file = "test_data/ingress_creation.json";
        let test_cases = vec![
            Testcase {
                name: String::from("Reject because it has been dropped from kubernetes 1.25.0"),
                fixture_file: String::from(request_file),
                expected_validation_result: false,
                settings: Settings {
                    kubernetes_version: semver::Version::parse("1.25.0").unwrap(),
                    deny_on_deprecation: true,
                },
            },
            Testcase {
                name: String::from("Reject because it has been deprected startiong from 1.14.0"),
                fixture_file: String::from(request_file),
                expected_validation_result: false,
                settings: Settings {
                    kubernetes_version: semver::Version::parse("1.19.0").unwrap(),
                    deny_on_deprecation: true,
                },
            },
            Testcase {
                name: String::from(
                    "Do not reject despite being deprected because of user settings",
                ),
                fixture_file: String::from(request_file),
                expected_validation_result: true,
                settings: Settings {
                    kubernetes_version: semver::Version::parse("1.19.0").unwrap(),
                    deny_on_deprecation: false,
                },
            },
            Testcase {
                name: String::from("Accept on a really old version of kubernetes"),
                fixture_file: String::from(request_file),
                expected_validation_result: true,
                settings: Settings {
                    kubernetes_version: semver::Version::parse("1.10.0").unwrap(),
                    deny_on_deprecation: false,
                },
            },
        ];

        for tc in &test_cases {
            let res = tc.eval(validate).unwrap();
            assert!(
                res.mutated_object.is_none(),
                "Something mutated with test case: {}",
                tc.name,
            );
        }

        Ok(())
    }
}
