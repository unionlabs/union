# Contributing to ibc-go

Thank you for considering making contributions to ibc-go! 🎉👍

## Code of conduct

This project and everyone participating in it is governed by ibc-go's [code of conduct](./CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code

## How can I contribute?

Contributing to this repository can mean many things such as participating in discussions or proposing code changes. To ensure a smooth workflow for all contributors, the general procedure for contributing has been established:

### Reporting bugs

If you find that something is not working as expected, please open an issue using the [bug report template](https://github.com/cosmos/ibc-go/blob/main/.github/ISSUE_TEMPLATE/bug-report.md) and provide as much information possible: how can the bug be reproduced? What's the expected behavior? What version is affected?

### Proposing improvements or new features

New features or improvements should be written in an issue using the [new feature template](https://github.com/cosmos/ibc-go/blob/main/.github/ISSUE_TEMPLATE/feature-request.md). Please include in the issue as many details as possible: what use case(s) would this new feature or improvement enable? Why are those use cases important or helpful? what user group would benefit? The team will evaluate and engage with you in a discussion of the proposal, which could have different outcomes:

- the core ibc-go team deciding to implement this feature and adding it to their planning,
- agreeing to support external contributors to implement it with the goal of merging it eventually in ibc-go,
- discarding the suggestion if deemed not aligned with the objectives of ibc-go;
- or proposing (in the case of applications or light clients) to be developed and maintained in a separate repository.

### Architecture Decision Records (ADR)

When proposing an architecture decision for the ibc-go, please create an [ADR](./docs/architecture/README.md) so further discussions can be made. We are following this process so all involved parties are in agreement before any party begins coding the proposed implementation. Please use the [ADR template](./docs/architecture/adr.template.md) to scaffold any new ADR. If you would like to see some examples of how these are written refer to ibc-go's [ADRs](./docs/architecture/). ADRs are solidified designs that will be implemented in ibc-go (and do not have a spec). They should document the architecture that will be built. Most design feedback should be gathered before the initial draft of the ADR. ADR's can/should be written for any design decisions we make which may be changed at some point in the future.

### Participating in discussions

New features or improvements are sometimes also debated in [discussions](https://github.com/cosmos/ibc-go/discussions). Sharing feedback or ideas there is very helpful for us. high level discussions that may get a lot of comments on a variety of different aspects, design aspects still being considered.

### Submitting pull requests

Unless you feel confident your change will be accepted (trivial bug fixes, code cleanup, etc) you should first create an issue to discuss your change with us. This lets us all discuss the design and proposed implementation of your change, which helps ensure your time is well spent and that your contribution will be accepted.

Looking for a good place to start contributing? The issue tracker is always the first place to go. Issues are triaged to categorize them:

- Check out some [`good first issue`s](https://github.com/cosmos/ibc-go/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22). These are issues whose scope of work should be pretty clearly specified and they are best suited for developers new to ibc-go (i.e. no deep knowledge of Cosmos SDK or ibc-go is required). For example, some of these issues may involve improving the logging, emitting new events or removing unused code.
- Or pick up a [`help wanted`](https://github.com/cosmos/ibc-go/issues?q=is%3Aopen+is%3Aissue+label%3A%22help+wanted%22) issue. These issues should be a bit more involved than the good first issues and the developer working on them would benefit from some familiarity already with the codebase. These types of issues may involve adding new (or extending the functionality of existing) gRPC endpoints, bumping the version of Cosmos SDK or Tendermint or fixing bugs.

If you would like to contribute, follow this process:

1. If the issue is a proposal, ensure that the proposal has been accepted.
2. Ensure that nobody else has already begun working on this issue. If they have, make sure to contact them to collaborate.
3. If nobody has been assigned for the issue and you would like to work on it, comment on the issue to inform the community of your intentions to begin work. Then we will be able to assign the issue to you, making it visible for others that this issue is being tackled. If you end up not creating a pull request for this issue, please comment on the issue as well, so that it can be assigned to somebody else.
4. Follow standard GitHub best practices: fork the repo, branch from the HEAD of `main`, make some commits, and submit a PR to `main`. For core developers working within the ibc-go repo, branches must be named with the convention `{moniker}/{issue#}-branch-name` to ensure a clear ownership of branches.
5. Feel free to submit the pull request in `Draft` mode, even if the work is not complete, as this indicates to the community you are working on something and allows them to provide comments early in the development process.
6. When the code is complete it can be marked `Ready for Review`.
7. Be sure to include a relevant changelog entry in the [Commit Message / Changelog Entry section of pull request description](https://github.com/cosmos/ibc-go/blob/main/.github/PULL_REQUEST_TEMPLATE.md#commit-message--changelog-entry) so that we can add changelog entry when merging the pull request. Please follow the [Conventional Commits specification](https://www.conventionalcommits.org/en/v1.0.0/) and use one of the commit types mentioned in the [Commit messages section of the pull request guidelines](./docs/dev/pull-requests.md#commit-messages).

Please make sure to check out our [Pull request guidelines](./docs/dev/pull-requests.md) for more information.

## Relevant development docs

- [Project structure](./docs/dev/project-structure.md)
- [Develoment setup](./docs/dev/development-setup.md)
- [Go style guide](./docs/dev/go-style-guide.md)
- [Documentation guide](./docs/README.md)
- [Writing tests](./testing/README.md)
- [Pull request guidelines](./docs/dev/pull-requests.md)
- [Release process](./docs/dev/release-management.md)
