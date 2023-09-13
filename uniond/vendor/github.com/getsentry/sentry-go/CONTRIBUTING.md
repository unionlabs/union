# Contributing to sentry-go

Hey, thank you if you're reading this, we welcome your contribution!

## Sending a Pull Request

Please help us save time when reviewing your PR by following this simple
process:

1. Is your PR a simple typo fix? Read no further, **click that green "Create
   pull request" button**!

2. For more complex PRs that involve behavior changes or new APIs, please
   consider [opening an **issue**][new-issue] describing the problem you're
   trying to solve if there's not one already.

   A PR is often one specific solution to a problem and sometimes talking about
   the problem unfolds new possible solutions. Remember we will be responsible
   for maintaining the changes later.

3. Fixing a bug and changing a behavior? Please add automated tests to prevent
   future regression.

4. Practice writing good commit messages. We have [commit
   guidelines][commit-guide].

5. We have [guidelines for PR submitters][pr-guide]. A short summary:

   - Good PR descriptions are very helpful and most of the time they include
     **why** something is done and why done in this particular way. Also list
     other possible solutions that were considered and discarded.
   - Be your own first reviewer. Make sure your code compiles and passes the
     existing tests.

[new-issue]: https://github.com/getsentry/sentry-go/issues/new/choose
[commit-guide]: https://develop.sentry.dev/code-review/#commit-guidelines
[pr-guide]: https://develop.sentry.dev/code-review/#guidelines-for-submitters

Please also read through our [SDK Development docs](https://develop.sentry.dev/sdk/).
It contains information about SDK features, expected payloads and best practices for
contributing to Sentry SDKs.

## Community

The public-facing channels for support and development of Sentry SDKs can be found on [Discord](https://discord.gg/Ww9hbqr).

## Testing

```console
$ go test
```

### Watch mode

Use: https://github.com/cespare/reflex

```console
$ reflex -g '*.go' -d "none" -- sh -c 'printf "\n"; go test'
```

### With data race detection

```console
$ go test -race
```

### Coverage

```console
$ go test -race -coverprofile=coverage.txt -covermode=atomic && go tool cover -html coverage.txt
```

## Linting

Lint with [`golangci-lint`](https://github.com/golangci/golangci-lint):

```console
$ golangci-lint run
```

## Release

1. Update `CHANGELOG.md` with new version in `vX.X.X` format title and list of changes.

   The command below can be used to get a list of changes since the last tag, with the format used in `CHANGELOG.md`:

   ```console
   $ git log --no-merges --format=%s $(git describe --abbrev=0).. | sed 's/^/- /'
   ```

2. Commit with `misc: vX.X.X changelog` commit message and push to `master`.

3. Let [`craft`](https://github.com/getsentry/craft) do the rest:

   ```console
   $ craft prepare X.X.X
   $ craft publish X.X.X
   ```
