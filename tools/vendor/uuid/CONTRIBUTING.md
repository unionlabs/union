Contributing to Uuid
---
[Contributing to Uuid]: #contributing-to-uuid

Thank you for your interest in contributing to the Uuid Project!

* [Feature Requests](#feature-requests)
* [Bug Reports](#bug-reports)
* [Pull Requests](#pull-requests)
* [Writing Documentation](#writing-documentation)
* [Issue Triage](#issue-triage)
* [Out-of-tree Contributions](#out-of-tree-contributions)
* [Helpful Links](#helpful-links)

For any questions, please make a post on [Discussions] or [users.rust-lang.org][u-r-l-o].

> All contributors need to follow our [Code of Conduct].

[Code of Conduct]: CODE_OF_CONDUCT.md

# Feature Requests
[Feature Requests]: #feature-requests

The `uuid` crate is stable so features that require breaking changes can't be
accepted. There are bound to be usability gaps that can be filled with non-breaking
features though!

If you have the chance, please [search existing issues], as there is a chance
that someone has already requested your feature.

File your feature request with a descriptive title, as this helps others find
your request.

You can request your feature by following [this link][Feature Request Link] and
filling it in. 

> We welcome pull requests for your own feature requests. Discussion should stay
on the relevant issue to make it easier to find.

## Adding new unstable dependencies

If a feature introduces a new unstable dependency then it needs to be private.
That means traits and types from the unstable library can't be visible in `uuid`'s
own public API.

To implement unstable traits, see the precedent set by the `zerocopy-unstable` feature.
It implements a public unstable trait on `Uuid`, but uses the `uuid_unstable` cfg flag.

[Feature Request Link]: https://github.com/uuid-rs/uuid/issues/new?template=Feature_request.md

# Bug Reports
[Bug Reports]: #bug-reports

While no one likes bugs, they are an unfortunate reality in software. Remember
we can't fix bugs we don't know about, so don't be shy about reporting.

If you have the chance, please [search existing issues], as there is a chance
that someone has already reported your error. This isn't strictly needed, as
sometimes you might not what exactly you are looking for.

File your issue with a descriptive title, as this helps others find your issue.

Reporting a bug is as easy as following [this link][Bug Report Link] and
filling it in.

Sometimes a backtrace may be needed. In that case, set `RUST_BACKTRACE`
environment variable to `1`. For example:

```bash
$ RUST_BACKTRACE=1 cargo build
```

> We welcome pull requests for your own bug reports, provided they have been
discussed.

[Bug Report Link]: https://github.com/uuid-rs/uuid/issues/new?template=Bug_report.md

# Pull Requests
[Pull Requests]: #pull-requests

Pull requests(PRs) are the primary mechanism we use to change Uuid. GitHub itself
has some [great documentation] on using the Pull Request feature. We use the
"fork and pull" model described [here][fnp], where contributors push changes to
their personal fork and create pull requests to bring those changes into the
source repository.

Unless the changes are fairly minor (like documentation changes or tiny
patches), we require PRs to relevant issues.

Please open PRs against the `main` branch.

When you feel that the PR is ready, please ping one of the maintainers so
they can review your changes.

[great documentation]: https://help.github.com/articles/about-pull-requests/
[fnp]: https://help.github.com/articles/about-collaborative-development-models/

# Writing Documentation
[Writing Documentation]: #writing-documentation

Documentation is an important part of Uuid. Lackluster or incorrect
documentation can cause headaches for the users of `uuid`. Therefore,
improvements to documentation are always welcome.

We follow the documentation style guidelines as given by [RFC 1574].

[RFC 1574]: https://github.com/rust-lang/rfcs/blob/main/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text

# Issue Triage
[Issue Triage]: #issue-triage

Sometimes, an issue might stay open even after the relevant bug has been fixed.
Other times, the bug report may become invalid. Or we may just forget about the
bug.

You can help to go through old bug reports and check if they are still valid.
You can follow [this link][lrus] to look for issues like this.

[lrus]: https://github.com/uuid-rs/uuid/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-asc

# Fuzzing
We use [`cargo fuzz`] to fuzz test various parts of `uuid`. See their guide
for more details on what fuzzing is and how to run the tests yourself.

# Helpful Links
[Helpful Links]: #helpful-links

For people new to Uuid, and just starting to contribute, or even for more
seasoned developers, some useful places to look for information are:

* The Wikipedia entry on [Universally Unique Identifier][wiki-uuid].
* [RFC 4122] which gives the specification of Uuids.

[wiki-uuid]: https://en.wikipedia.org/wiki/Universally_unique_identifier
[RFC 4122]: https://www.ietf.org/rfc/rfc4122.txt

[u-r-l-o]: https://users.rust-lang.org
[Discussions]: https://github.com/uuid-rs/uuid/discussions
[search existing issues]: https://github.com/uuid-rs/uuid/search?q=&type=Issues&utf8=%E2%9C%93
[`cargo fuzz`]: https://rust-fuzz.github.io/book/cargo-fuzz.html
