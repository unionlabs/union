# ADR 047: Extend Upgrade Plan

## Changelog

* Nov, 23, 2021: Initial Draft
* May, 16, 2023: Proposal ABANDONED. `pre_run` and `post_run` are not necessary anymore and adding the `artifacts` brings minor benefits.

## Status

ABANDONED

## Abstract

This ADR expands the existing x/upgrade `Plan` proto message to include new fields for defining pre-run and post-run processes within upgrade tooling.
It also defines a structure for providing downloadable artifacts involved in an upgrade.

## Context

The `upgrade` module in conjunction with Cosmovisor are designed to facilitate and automate a blockchain's transition from one version to another.

Users submit a software upgrade governance proposal containing an upgrade `Plan`.
The [Plan](https://github.com/cosmos/cosmos-sdk/blob/v0.44.5/proto/cosmos/upgrade/v1beta1/upgrade.proto#L12) currently contains the following fields:

* `name`: A short string identifying the new version.
* `height`: The chain height at which the upgrade is to be performed.
* `info`: A string containing information about the upgrade.

The `info` string can be anything.
However, Cosmovisor will try to use the `info` field to automatically download a new version of the blockchain executable.
For the auto-download to work, Cosmovisor expects it to be either a stringified JSON object (with a specific structure defined through documentation), or a URL that will return such JSON.
The JSON object identifies URLs used to download the new blockchain executable for different platforms (OS and Architecture, e.g. "linux/amd64").
Such a URL can either return the executable file directly or can return an archive containing the executable and possibly other assets.

If the URL returns an archive, it is decompressed into `{DAEMON_HOME}/cosmovisor/{upgrade name}`.
Then, if `{DAEMON_HOME}/cosmovisor/{upgrade name}/bin/{DAEMON_NAME}` does not exist, but `{DAEMON_HOME}/cosmovisor/{upgrade name}/{DAEMON_NAME}` does, the latter is copied to the former.
If the URL returns something other than an archive, it is downloaded to `{DAEMON_HOME}/cosmovisor/{upgrade name}/bin/{DAEMON_NAME}`.

If an upgrade height is reached and the new version of the executable version isn't available, Cosmovisor will stop running.

Both `DAEMON_HOME` and `DAEMON_NAME` are [environment variables used to configure Cosmovisor](https://github.com/cosmos/cosmos-sdk/blob/cosmovisor/v1.0.0/cosmovisor/README.md#command-line-arguments-and-environment-variables).

Currently, there is no mechanism that makes Cosmovisor run a command after the upgraded chain has been restarted.

The current upgrade process has this timeline:

1. An upgrade governance proposal is submitted and approved.
1. The upgrade height is reached.
1. The `x/upgrade` module writes the `upgrade_info.json` file.
1. The chain halts.
1. Cosmovisor backs up the data directory (if set up to do so).
1. Cosmovisor downloads the new executable (if not already in place).
1. Cosmovisor executes the `${DAEMON_NAME} pre-upgrade`.
1. Cosmovisor restarts the app using the new version and same args originally provided.

## Decision

### Protobuf Updates

We will update the `x/upgrade.Plan` message for providing upgrade instructions.
The upgrade instructions will contain a list of artifacts available for each platform.
It allows for the definition of a pre-run and post-run commands.
These commands are not consensus guaranteed; they will be executed by Cosmosvisor (or other) during its upgrade handling.

```protobuf
message Plan {
  // ... (existing fields)

  UpgradeInstructions instructions = 6;
}
```

The new `UpgradeInstructions instructions` field MUST be optional.

```protobuf
message UpgradeInstructions {
  string pre_run              = 1;
  string post_run             = 2;
  repeated Artifact artifacts = 3;
  string description          = 4;
}
```

All fields in the `UpgradeInstructions` are optional.

* `pre_run` is a command to run prior to the upgraded chain restarting.
  If defined, it will be executed after halting and downloading the new artifact but before restarting the upgraded chain.
  The working directory this command runs from MUST be `{DAEMON_HOME}/cosmovisor/{upgrade name}`.
  This command MUST behave the same as the current [pre-upgrade](https://github.com/cosmos/cosmos-sdk/blob/v0.44.5/docs/migrations/pre-upgrade.md) command.
  It does not take in any command-line arguments and is expected to terminate with the following exit codes:

  | Exit status code | How it is handled in Cosmosvisor                                                                                    |
  |------------------|---------------------------------------------------------------------------------------------------------------------|
  | `0`              | Assumes `pre-upgrade` command executed successfully and continues the upgrade.                                      |
  | `1`              | Default exit code when `pre-upgrade` command has not been implemented.                                              |
  | `30`             | `pre-upgrade` command was executed but failed. This fails the entire upgrade.                                       |
  | `31`             | `pre-upgrade` command was executed but failed. But the command is retried until exit code `1` or `30` are returned. |
  If defined, then the app supervisors (e.g. Cosmovisor) MUST NOT run `app pre-run`.
* `post_run` is a command to run after the upgraded chain has been started. If defined, this command MUST be only executed at most once by an upgrading node.
  The output and exit code SHOULD be logged but SHOULD NOT affect the running of the upgraded chain.
  The working directory this command runs from MUST be `{DAEMON_HOME}/cosmovisor/{upgrade name}`.
* `artifacts` define items to be downloaded.
  It SHOULD have only one entry per platform.
* `description` contains human-readable information about the upgrade and might contain references to external resources.
  It SHOULD NOT be used for structured processing information.

```protobuf
message Artifact {
  string platform      = 1;
  string url           = 2;
  string checksum      = 3;
  string checksum_algo = 4;
}
```

* `platform` is a required string that SHOULD be in the format `{OS}/{CPU}`, e.g. `"linux/amd64"`.
  The string `"any"` SHOULD also be allowed.
  An `Artifact` with a `platform` of `"any"` SHOULD be used as a fallback when a specific `{OS}/{CPU}` entry is not found.
  That is, if an `Artifact` exists with a `platform` that matches the system's OS and CPU, that should be used;
  otherwise, if an `Artifact` exists with a `platform` of `any`, that should be used;
  otherwise no artifact should be downloaded.
* `url` is a required URL string that MUST conform to [RFC 1738: Uniform Resource Locators](https://www.ietf.org/rfc/rfc1738.txt).
  A request to this `url` MUST return either an executable file or an archive containing either `bin/{DAEMON_NAME}` or `{DAEMON_NAME}`.
  The URL should not contain checksum - it should be specified by the `checksum` attribute.
* `checksum` is a checksum of the expected result of a request to the `url`.
  It is not required, but is recommended.
  If provided, it MUST be a hex encoded checksum string.
  Tools utilizing these `UpgradeInstructions` MUST fail if a `checksum` is provided but is different from the checksum of the result returned by the `url`.
* `checksum_algo` is a string identify the algorithm used to generate the `checksum`.
  Recommended algorithms: `sha256`, `sha512`.
  Algorithms also supported (but not recommended): `sha1`, `md5`.
  If a `checksum` is provided, a `checksum_algo` MUST also be provided.

A `url` is not required to contain a `checksum` query parameter.
If the `url` does contain a `checksum` query parameter, the `checksum` and `checksum_algo` fields MUST also be populated, and their values MUST match the value of the query parameter.
For example, if the `url` is `"https://example.com?checksum=md5:d41d8cd98f00b204e9800998ecf8427e"`, then the `checksum` field must be `"d41d8cd98f00b204e9800998ecf8427e"` and the `checksum_algo` field must be `"md5"`.

### Upgrade Module Updates

If an upgrade `Plan` does not use the new `UpgradeInstructions` field, existing functionality will be maintained.
The parsing of the `info` field as either a URL or `binaries` JSON will be deprecated.
During validation, if the `info` field is used as such, a warning will be issued, but not an error.

We will update the creation of the `upgrade-info.json` file to include the `UpgradeInstructions`.

We will update the optional validation available via CLI to account for the new `Plan` structure.
We will add the following validation:

1.  If `UpgradeInstructions` are provided:
    1.  There MUST be at least one entry in `artifacts`.
    1.  All of the `artifacts` MUST have a unique `platform`.
    1.  For each `Artifact`, if the `url` contains a `checksum` query parameter:
        1. The `checksum` query parameter value MUST be in the format of `{checksum_algo}:{checksum}`.
        1. The `{checksum}` from the query parameter MUST equal the `checksum` provided in the `Artifact`.
        1. The `{checksum_algo}` from the query parameter MUST equal the `checksum_algo` provided in the `Artifact`.
1.  The following validation is currently done using the `info` field. We will apply similar validation to the `UpgradeInstructions`.
    For each `Artifact`:
    1.  The `platform` MUST have the format `{OS}/{CPU}` or be `"any"`.
    1.  The `url` field MUST NOT be empty.
    1.  The `url` field MUST be a proper URL.
    1.  A `checksum` MUST be provided either in the `checksum` field or as a query parameter in the `url`.
    1.  If the `checksum` field has a value and the `url` also has a `checksum` query parameter, the two values MUST be equal.
    1.  The `url` MUST return either a file or an archive containing either `bin/{DAEMON_NAME}` or `{DAEMON_NAME}`.
    1.  If a `checksum` is provided (in the field or as a query param), the checksum of the result of the `url` MUST equal the provided checksum.

Downloading of an `Artifact` will happen the same way that URLs from `info` are currently downloaded.

### Cosmovisor Updates

If the `upgrade-info.json` file does not contain any `UpgradeInstructions`, existing functionality will be maintained.

We will update Cosmovisor to look for and handle the new `UpgradeInstructions` in `upgrade-info.json`.
If the `UpgradeInstructions` are provided, we will do the following:

1.  The `info` field will be ignored.
1.  The `artifacts` field will be used to identify the artifact to download based on the `platform` that Cosmovisor is running in.
1.  If a `checksum` is provided (either in the field or as a query param in the `url`), and the downloaded artifact has a different checksum, the upgrade process will be interrupted and Cosmovisor will exit with an error.
1.  If a `pre_run` command is defined, it will be executed at the same point in the process where the `app pre-upgrade` command would have been executed.
    It will be executed using the same environment as other commands run by Cosmovisor.
1.  If a `post_run` command is defined, it will be executed after executing the command that restarts the chain.
    It will be executed in a background process using the same environment as the other commands.
    Any output generated by the command will be logged.
    Once complete, the exit code will be logged.

We will deprecate the use of the `info` field for anything other than human readable information.
A warning will be logged if the `info` field is used to define the assets (either by URL or JSON).

The new upgrade timeline is very similar to the current one. Changes are in bold:

1. An upgrade governance proposal is submitted and approved.
1. The upgrade height is reached.
1. The `x/upgrade` module writes the `upgrade_info.json` file **(now possibly with `UpgradeInstructions`)**.
1. The chain halts.
1. Cosmovisor backs up the data directory (if set up to do so).
1. Cosmovisor downloads the new executable (if not already in place).
1. Cosmovisor executes **the `pre_run` command if provided**, or else the `${DAEMON_NAME} pre-upgrade` command.
1. Cosmovisor restarts the app using the new version and same args originally provided.
1. **Cosmovisor immediately runs the `post_run` command in a detached process.**

## Consequences

### Backwards Compatibility

Since the only change to existing definitions is the addition of the `instructions` field to the `Plan` message, and that field is optional, there are no backwards incompatibilities with respects to the proto messages.
Additionally, current behavior will be maintained when no `UpgradeInstructions` are provided, so there are no backwards incompatibilities with respects to either the upgrade module or Cosmovisor.

### Forwards Compatibility

In order to utilize the `UpgradeInstructions` as part of a software upgrade, both of the following must be true:

1.  The chain must already be using a sufficiently advanced version of the Cosmos SDK.
1.  The chain's nodes must be using a sufficiently advanced version of Cosmovisor.

### Positive

1.  The structure for defining artifacts is clearer since it is now defined in the proto instead of in documentation.
1.  Availability of a pre-run command becomes more obvious.
1.  A post-run command becomes possible.

### Negative

1.  The `Plan` message becomes larger. This is negligible because A) the `x/upgrades` module only stores at most one upgrade plan, and B) upgrades are rare enough that the increased gas cost isn't a concern.
1.  There is no option for providing a URL that will return the `UpgradeInstructions`.
1.  The only way to provide multiple assets (executables and other files) for a platform is to use an archive as the platform's artifact.

### Neutral

1. Existing functionality of the `info` field is maintained when the `UpgradeInstructions` aren't provided.

## Further Discussions

1.  [Draft PR #10032 Comment](https://github.com/cosmos/cosmos-sdk/pull/10032/files?authenticity_token=pLtzpnXJJB%2Fif2UWiTp9Td3MvRrBF04DvjSuEjf1azoWdLF%2BSNymVYw9Ic7VkqHgNLhNj6iq9bHQYnVLzMXd4g%3D%3D&file-filters%5B%5D=.go&file-filters%5B%5D=.proto#r698708349):
    Consider different names for `UpgradeInstructions instructions` (either the message type or field name).
1.  [Draft PR #10032 Comment](https://github.com/cosmos/cosmos-sdk/pull/10032/files?authenticity_token=pLtzpnXJJB%2Fif2UWiTp9Td3MvRrBF04DvjSuEjf1azoWdLF%2BSNymVYw9Ic7VkqHgNLhNj6iq9bHQYnVLzMXd4g%3D%3D&file-filters%5B%5D=.go&file-filters%5B%5D=.proto#r754655072):
    1.  Consider putting the `string platform` field inside `UpgradeInstructions` and make `UpgradeInstructions` a repeated field in `Plan`.
    1.  Consider using a `oneof` field in the `Plan` which could either be `UpgradeInstructions` or else a URL that should return the `UpgradeInstructions`.
    1.  Consider allowing `info` to either be a JSON serialized version of `UpgradeInstructions` or else a URL that returns that.
1.  [Draft PR #10032 Comment](https://github.com/cosmos/cosmos-sdk/pull/10032/files?authenticity_token=pLtzpnXJJB%2Fif2UWiTp9Td3MvRrBF04DvjSuEjf1azoWdLF%2BSNymVYw9Ic7VkqHgNLhNj6iq9bHQYnVLzMXd4g%3D%3D&file-filters%5B%5D=.go&file-filters%5B%5D=.proto#r755462876):
    Consider not including the `UpgradeInstructions.description` field, using the `info` field for that purpose instead.
1.  [Draft PR #10032 Comment](https://github.com/cosmos/cosmos-sdk/pull/10032/files?authenticity_token=pLtzpnXJJB%2Fif2UWiTp9Td3MvRrBF04DvjSuEjf1azoWdLF%2BSNymVYw9Ic7VkqHgNLhNj6iq9bHQYnVLzMXd4g%3D%3D&file-filters%5B%5D=.go&file-filters%5B%5D=.proto#r754643691):
    Consider allowing multiple artifacts to be downloaded for any given `platform` by adding a `name` field to the `Artifact` message.
1.  [PR #10502 Comment](https://github.com/cosmos/cosmos-sdk/pull/10602#discussion_r781438288)
    Allow the new `UpgradeInstructions` to be provided via URL.
1.  [PR #10502 Comment](https://github.com/cosmos/cosmos-sdk/pull/10602#discussion_r781438288)
    Allow definition of a `signer` for assets (as an alternative to using a `checksum`).

## References

* [Current upgrade.proto](https://github.com/cosmos/cosmos-sdk/blob/v0.44.5/proto/cosmos/upgrade/v1beta1/upgrade.proto)
* [Upgrade Module README](https://github.com/cosmos/cosmos-sdk/blob/v0.44.5/x/upgrade/spec/README.md)
* [Cosmovisor README](https://github.com/cosmos/cosmos-sdk/blob/cosmovisor/v1.0.0/cosmovisor/README.md)
* [Pre-upgrade README](https://github.com/cosmos/cosmos-sdk/blob/v0.44.5/docs/migrations/pre-upgrade.md)
* [Draft/POC PR #10032](https://github.com/cosmos/cosmos-sdk/pull/10032)
* [RFC 1738: Uniform Resource Locators](https://www.ietf.org/rfc/rfc1738.txt)
