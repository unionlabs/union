# Security

## Reporting Security Issues

<!-- This section is the copy of the organization security policy: https://github.com/getsentry/.github/blob/main/SECURITY.md -->

If you've found a security issue in Sentry or in our supported SDKs, you can submit your report to `security[@]sentry.io` via email.

Please include as much information as possible in your report to better help us understand and resolve the issue:

- Where the security issue exists (ie. Sentry SaaS, a Sentry-supported SDK, infrastructure, etc.)
- The type of issue (ex. SQL injection, cross-site scripting, missing authorization, etc.)
- Full paths or links to the source files where the security issue exists, if possible
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof of concept or exploit code, if available

If you need to encrypt sensitive information sent to us, please use [our PGP key](https://pgp.mit.edu/pks/lookup?op=vindex&search=0x641D2F6C230DBE3B):

```
E406 C27A E971 6515 A1B1 ED86 641D 2F6C 230D BE3B
```


## Dependency Update Policy

`sentry-go` has a number of external dependencies. While we try to keep that number low, some of those dependencies may contain security issues, and thus have to be updated.

In order to stay aligned with our [compatibility philosophy](https://develop.sentry.dev/sdk/philosophy/#compatibility-is-king), we take into account the category of the affected dependency and adhere to the following guidelines:

* **Core dependencies**: If there's a security issue in one of the core SDK dependencies (for example, `golang.org/x/sys`), we aim at updating it to a patched version in the next `sentry-go` release, assuming the patched version is available.

* **Integration dependencies**: If a security issue is discovered in one of our integration dependencies (for example, `gin`, `echo`, or `negroni`), it is the responsibility of the end user to make sure that those modules are updated in a timely manner in their applications.

  * Those frameworks and libraries are not used for the core SDK functionality, and are only present in the final dependency tree if the target application uses them as well, thanks to the [module graph pruning and lazy module loading](https://go.dev/ref/mod#graph-pruning).

  * If the vulnerable dependency is bumped to a newer (patched) version in the target application, `sentry-go` will also use it (thanks to the [minimal version selection](https://go.dev/ref/mod#minimal-version-selection) algorithm), and not the minimal required version as specified in `go.mod` of `sentry-go`.
