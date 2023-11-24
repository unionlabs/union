# Coordinated Vulnerability Disclosure Policy

The Cosmos ecosystem believes that strong security is a blend of highly
technical security researchers who care about security and the forward
progression of the ecosystem and the attentiveness and openness of Cosmos core
contributors to help continually secure our operations.

> **IMPORTANT**: *DO NOT* open public issues on this repository for security
> vulnerabilities.

## Scope

| Scope                 |
|-----------------------|
| last release (tagged) |
| main branch           |

The latest **release tag** of this repository is supported for security updates
as well as the **main** branch. Security vulnerabilities should be reported if
the vulnerability can be reproduced on either one of those.

## Reporting a Vulnerability

| Reporting methods                                             |
|---------------------------------------------------------------|
| [GitHub Private Vulnerability Reporting][gh-private-advisory] |
| [HackerOne bug bounty program][h1]                            |

All security vulnerabilities can be reported under GitHub's [Private
vulnerability reporting][gh-private-advisory] system. This will open a private
issue for the developers. Try to fill in as much of the questions as possible.
If you are not familiar with the CVSS system for assessing vulnerabilities, just
use the Low/High/Critical severity ratings. A partially filled in report for a
critical vulnerability is still better than no report at all.

Vulnerabilities associated with the **Go, Rust or Protobuf code** of the
repository may be eligible for a [bug bounty][h1]. Please see the bug bounty
page for more details on submissions and rewards. If you think the vulnerability
is eligible for a payout, **report on HackerOne first**.

Vulnerabilities in services and their source codes (JavaScript, web page, Google
Workspace) are not in scope for the bug bounty program, but they are welcome to
be reported in GitHub.

### Guidelines

We require that all researchers:

* Abide by this policy to disclose vulnerabilities, and avoid posting
  vulnerability information in public places, including GitHub, Discord,
  Telegram, and Twitter.
* Make every effort to avoid privacy violations, degradation of user experience,
  disruption to production systems (including but not limited to the Cosmos
  Hub), and destruction of data.
* Keep any information about vulnerabilities that you’ve discovered confidential
  between yourself and the Cosmos engineering team until the issue has been
  resolved and disclosed.
* Avoid posting personally identifiable information, privately or publicly.

If you follow these guidelines when reporting an issue to us, we commit to:

* Not pursue or support any legal action related to your research on this
  vulnerability
* Work with you to understand, resolve and ultimately disclose the issue in a
  timely fashion

### More information

* See [TIMELINE.md] for an example timeline of a disclosure.
* See [DISCLOSURE.md] to see more into the inner workings of the disclosure
  process.
* See [EXAMPLES.md] for some of the examples that we are interested in for the
  bug bounty program.

[gh-private-advisory]: /../../security/advisories/new
[h1]: https://hackerone.com/cosmos
[TIMELINE.md]: https://github.com/cosmos/security/blob/main/TIMELINE.md
[DISCLOSURE.md]: https://github.com/cosmos/security/blob/main/DISCLOSURE.md
[EXAMPLES.md]: https://github.com/cosmos/security/blob/main/EXAMPLES.md
