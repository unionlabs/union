---
sidebar_position: 1
---

# SDK Migrations

To smoothen the update to the latest stable release, the SDK includes a CLI command for hard-fork migrations (under the `<appd> genesis migrate` subcommand). 
Additionally, the SDK includes in-place migrations for its core modules. These in-place migrations are useful to migrate between major releases.

* Hard-fork migrations are supported from the last major release to the current one.
* In-place module migrations are supported from the last two major releases to the current one.

Migration from a version older than the last two major releases is not supported.

When migrating from a previous version, refer to the [`UPGRADING.md`](./02-upgrading.md) and the `CHANGELOG.md` of the version you are migrating to.
