#!/usr/bin/env bats

@test "Reject because deprecated" {
  run kwctl run \
    --request-path test_data/ingress_creation.json \
    --settings-json '{ "kubernetes_version" : "1.19.0", "deny_on_deprecation_default": true }' \
    annotated-policy.wasm

  # this prints the output when one the checks below fails
  echo "output = ${output}"

  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*"allowed":false.*') -ne 0 ]
}

@test "By default deny_on_deprecation is enabled" {
  run kwctl run \
    --request-path test_data/ingress_creation.json \
    --settings-json '{ "kubernetes_version" : "1.19.0" }' \
    annotated-policy.wasm

  # this prints the output when one the checks below fails
  echo "output = ${output}"

  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*"allowed":false.*') -ne 0 ]
}

@test "Handle deny_on_deprecation set to false" {
  run kwctl run \
    --request-path test_data/ingress_creation.json \
    --settings-json '{ "kubernetes_version" : "1.19.0", "deny_on_deprecation": false }' \
    annotated-policy.wasm

  # this prints the output when one the checks below fails
  echo "output = ${output}"

  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
}

@test "Reject because dropped" {
  run kwctl run \
    --request-path test_data/ingress_creation.json \
    --settings-json '{ "kubernetes_version" : "1.25.0" }' \
    annotated-policy.wasm

  # this prints the output when one the checks below fails
  echo "output = ${output}"

  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*"allowed":false.*') -ne 0 ]
}

@test "Accept on a really old version of kubernetes" {
  run kwctl run \
    --request-path test_data/ingress_creation.json \
    --settings-json '{ "kubernetes_version" : "1.10.0" }' \
    annotated-policy.wasm

  # this prints the output when one the checks below fails
  echo "output = ${output}"

  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
}


