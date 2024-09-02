# Welcome to Union

Thank you for investing your time in contributing to our project!

In this guide, you will get an overview of the contribution workflow from opening an issue, creating a PR, reviewing, and merging the PR.

## Before you start working on a PR

Before starting work on any code changes, make sure to start by creating an [issue](https://github.com/unionlabs/union/issues) first. Once you've created an issue you can start discussing the implementation, challenges, and work with core members before starting on code contributions.

For more complex conversations, use the [discussions](https://github.com/unionlabs/union/discussions). Usually, a discussion becomes one or more issues.

We evaluate the need for a PR based on:

1. Severity of the issue (bug or feature request),
1. Maintainability: will this become a burden for little gain, or add value?
1. Can the core team understand the code additions being made, and maintain them, or will they rely on you in the future?

## Working on a PR

Make sure to check the ARCHITECTURE.md document to get a feel for the repository structure. It explains our build system more in-depth. If you want to run approximately the same tests as CI does, run `nix flake check`. For this, you will need to install [nix](https://zero-to-nix.com/start/install).

### Adding new packages

Sometimes you might need to extend the CI. For this always use nix. You'll need to keep a few things into account:

1. Your additions must ensure compatibility with our supported targets:

```nix
systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
```

This means you cannot make changes that will only work on `x86_64-linux`, such as checking in binaries.

2. Check out existing Nix files and see if you can import packages and reuse definitions instead of redefining build steps.

If your new derivation adds packages and checks, it will be picked up automatically by CI. No need to edit anything in the workflows.

## Opening the PR

The PR should have a proper description, ensuring that code reviewers do not need to ask questions about the "why" of the PR. Your code will be reviewed once all CI checks pass. If you would like early feedback, mark it as a draft PR in the GitHub UI and ping us.

## Reviews

If you'd like your code to be merged quickly, address review comments as quickly as possible. We prioritize having as few open PRs as possible, so we'll do our part in reviewing quickly.

Finally, remain polite and friendly. Reviewing can feel harsh, but it is a learning opportunity most of all.

## Post-Merge

You'll be an official contributor and mentioned in the release, as well as visible in GitHub's analytics.
