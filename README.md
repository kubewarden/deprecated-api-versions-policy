[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Kubewarden policy deprecated-api-versions

## Description

This policy detects usage of Kubernetes resources that have been deprecated
or removed.

At deployment time, the operator must provide a Kubernetes version to use when
looking for deprecation/removal objects.
This is done via the `kubernetes_version` attribute.

For example, given the following configuration:

```yaml
kubernetes_version: "1.24.2"
```

The policy will detect all the Kubernetes resources that are deprecated or removed
starting from the Kubernetes version `1.24.2`.

## Keeping up with Kubernetes deprecations

Kubernetes deprecation evolve over the time. As soon as new deprecations are
added this policy will be updated.

Currently, this policy is aware of deprecations introduced up to the Kubernetes **1.32.0** release.

## Deprecated but not yet removed resources

By default the policy will prevent the usage of Kubernetes resources that are
already deprecated, but not yet removed.

This behaviour can be changed via the `deny_on_deprecation` setting.

For example, let's assume we are using an old version of Kubernetes like
`1.19.3` and someone is attempting to create a `extensions/v1beta1/Ingress`
object.

This kind of resource has been deprecated starting from `v1.14.0` of Kubernetes,
but it has been removed starting from version `v1.22.0`.

Given the following configuration:

```yaml
kubernetes_version: "1.19.0"
deny_on_deprecation: false
```

The `extensions/v1beta1/Ingress` object will be accepted inside of the cluster.

On the other hand, it would be blocked with this configuration:

```yaml
kubernetes_version: "1.19.0"
deny_on_deprecation: true # note: this is set to true by default
```
