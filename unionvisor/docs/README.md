{% if doc_comment %}
{{ doc_comment }}
{% else %}

<!-- This doc is a template for the final README.md, which is generated with docgen -->

{% endif %}

# Unionvisor

Unionvisor is a utility for managing [`uniond`](../uniond) deployments. It manages upgrade lifecycles and integrates well with NixOS.

## NixOS Configuration

An example flake.nix configuration can be found in [`usage.nix`](./usage.nix):

```nix
{% include "usage.nix" %}
```

The configuration creates a production-ready machine running a validator under unionvisor, using the unionbundle. Bundles are packages that contain historic `uniond` binaries. They are capable of syncing a chain from zero and performing upgrades, effectively [bootstrapping](<https://en.wikipedia.org/wiki/Bootstrapping_(compilers)>) and verifying the full history.
