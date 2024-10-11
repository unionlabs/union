# Contributing

Thank you for considering making contributions to Wasmd!

Contributing to this repo can mean many things, such as participating in
discussion or proposing code changes. To ensure a smooth workflow for all
contributors, the general procedure for contributing has been established:

1. Start by browsing [new issues](https://github.com/CosmWasm/wasmd/issues).
   * Looking for a good place to start contributing? How about checking out some [good first issues](https://github.com/CosmWasm/wasmd/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22) or [bugs](https://github.com/CosmWasm/wasmd/issues?q=is%3Aopen+is%3Aissue+label%3Abug)?
2. Determine whether a GitHub issue or discussion is more appropriate for your needs:
   1. If the issue you want addressed is a specific proposal or a bug, then open a [new issue](https://github.com/CosmWasm/wasmd/issues/new).
   2. Review existing [issues](https://github.com/CosmWasm/wasmd/issues) to find an issue you'd like to help with.
3. Participate in thoughtful discussion on that issue.
4. If you would like to contribute:
   1. Ensure that the proposal has been accepted.
   2. Ensure that nobody else has already begun working on this issue. If they have,
      make sure to contact them to collaborate.
   3. If nobody has been assigned for the issue and you would like to work on it,
      make a comment on the issue to inform the community of your intentions
      to begin work.
5. To submit your work as a contribution to the repository follow standard GitHub best practices. See [pull request guideline](#pull-requests) below.

**Note:** For very small or blatantly obvious problems such as typos, you are
not required to an open issue to submit a PR, but be aware that for more complex
problems/features, if a PR is opened before an adequate design discussion has
taken place in a GitHub issue, that PR runs a high likelihood of being rejected.

## Development Procedure

* The latest state of development is on `main`.
* `main` must never fail `make lint test test-race`.
* No `--force` onto `main` (except when reverting a broken commit, which should seldom happen).
* Create a branch to start work:
    * Fork the repo (core developers must create a branch directly in the Wasmd repo),
    branch from the HEAD of `main`, make some commits, and submit a PR to `main`.
    * For core developers working within the `wasmd` repo, follow branch name conventions to ensure a clear
    ownership of branches: `{issue#}-branch-name`.
    * See [Branching Model](#branching-model-and-release) for more details.
* Be sure to run `make format` before every commit. The easiest way
  to do this is have your editor run it for you upon saving a file (most of the editors
  will do it anyway using a pre-configured setup of the programming language mode).
* Follow the [CODING GUIDELINES](CODING_GUIDELINES.md), which defines criteria for designing and coding a software.

Code is merged into main through pull request procedure.

### Testing

Tests can be executed by running `make test` at the top level of the wasmd repository.

### Pull Requests

Before submitting a pull request:

* merge the latest main `git merge origin/main`,
* run `make lint test` to ensure that all checks and tests pass.

Then:

1. If you have something to show, **start with a `Draft` PR**. It's good to have early validation of your work and we highly recommend this practice. A Draft PR also indicates to the community that the work is in progress.
   Draft PRs also helps the core team provide early feedback and ensure the work is in the right direction.
2. When the code is complete, change your PR from `Draft` to `Ready for Review`.
3. Be sure to include a relevant changelog entry in the `Unreleased` section of `CHANGELOG.md` (see file for log format). The entry should be on top of all others changes in the section.

PRs name should start upper case.
Additionally, each PR should only address a single issue.

NOTE: when merging, GitHub will squash commits and rebase on top of the main.

## Protobuf

We use [Protocol Buffers](https://developers.google.com/protocol-buffers) along with [gogoproto](https://github.com/cosmos/gogoproto) to generate code for use in Wasmd.

For deterministic behavior around Protobuf tooling, everything is containerized using Docker. Make sure to have Docker installed on your machine, or head to [Docker's website](https://docs.docker.com/get-docker/) to install it.

For formatting code in `.proto` files, you can run `make proto-format` command.

For linting we use [buf](https://buf.build/). You can use the commands `make proto-lint` to lint your proto files.

To generate the protobuf stubs, you can run `make proto-gen`.

We also added the `make proto-all` command to run all the above commands sequentially.

In order for imports to properly compile in your IDE, you may need to manually set your protobuf path in your IDE's workspace settings/config.

For example, in vscode your `.vscode/settings.json` should look like:

```json
{
    "protoc": {
        "options": [
        "--proto_path=${workspaceRoot}/proto",
        ]
    }
}
```

## Branching Model and Release

User-facing repos should adhere to the trunk based development branching model: https://trunkbaseddevelopment.com. User branches should start with a user name, example: `{moniker}/{issue#}-branch-name`.

Wasmd utilizes [semantic versioning](https://semver.org/).

### PR Targeting

Ensure that you base and target your PR on the `main` branch.

All feature additions and all bug fixes must be targeted against `main`. Exception is for bug fixes which are only related to a released version. In that case, the related bug fix PRs must target against the release branch.

If needed, we backport a commit from `main` to a release branch (excluding consensus breaking feature, API breaking and similar).