# Welcome to Union

Thank you for investing your time in contributing to our project!

In this guide, you will get an overview of the contribution workflow from opening an issue, creating a PR, reviewing, and merging the PR.

## Before you start working on a PR

Before starting work on any code changes, make sure to start by creating an [issue](https://github.com/unionlabs/union/issues) first. Once you've created an issue you can start discussing the implementation, challenges, and work with core members before starting on code contributions.

For more complex conversations, use the [discussions](https://github.com/unionlabs/union/discussions). Usually, a discussion becomes one or more issues.

We evaluate the need for a PR based on:

1. Severity of the issue (bug or feature request),
2. Maintainability: will this become a burden for little gain, or add value?
3. Can the core team understand the code additions being made, and maintain them, or will they rely on you in the future?

## Working on a PR

Make sure to check the ARCHITECTURE.md document to get a feel for the repository structure. It explains our build system more in-depth. If you want to run approximately the same tests as CI does, run `nix flake check`. For this, you will need to install [nix](https://zero-to-nix.com/start/install).

> \[!IMPORTANT\]
> Currently, only nix versions \<=2.18 or >=2.25 work reliably in our repository. If you're on macOS using an orbstack NixOS vm, be sure to create it with NixOS 24.05 as it ships with nix 2.18. NixOS 24.11 ships with nix 2.24, which breaks certain workflows in our repository.

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

The PR should have a proper description, ensuring that code reviewers do not need to ask questions about the "why" of the PR. If the PR has a related issue, be sure to mention it in the description as well with a [closing keyword](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue#linking-a-pull-request-to-an-issue-using-a-keyword) such as `closes #1337`. Your code will be reviewed once all CI checks pass. If you would like early feedback, mark it as a draft PR in the GitHub UI and ping us.

Some things to keep in mind while working on a PR:

- All commits must follow [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#summary). This is enforced by our CI.
- Your commits must be [signed](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits). This is also enforced by our CI.
  - Aside from the requirement for this repository, it is good practice to sign your commits, otherwise anyone can trivially impersonate you on github and you can get [rekt](https://github.com/KaiserKarel/rekt).
- If you have `nix` installed, ensure you run `nix fmt` and `nix build .#checks.your-architecture.spellcheck` before pushing. These are both checked in CI, and if they're checked locally first it will make the lifecycle of your PR much faster.
- We do not use merge commits in this repository. If you have conflicts in your branch, you must rebase it on top of latest main (as opposed to merging main into your branch). This results in a much cleaner git history. This is *not* enforced by our CI, but your PR will not be approved if it contains merge commits.

## Reviews

If you'd like your code to be merged quickly, address review comments as quickly as possible. We prioritize having as few open PRs as possible, so we'll do our part in reviewing quickly.

Finally, remain polite and friendly. Reviewing can feel harsh, but it is a learning opportunity most of all.

## Post-Merge

You'll be an official contributor and mentioned in the release, as well as visible in GitHub's analytics.
