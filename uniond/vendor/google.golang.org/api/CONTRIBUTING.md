# Contributing

1. Sign one of the
[contributor license agreements](#contributor-license-agreements) below.
1. [Install Go](https://golang.org/doc/install).
1. Clone the repo:

    `git clone https://github.com/googleapis/google-api-go-client`
1. Change into the checked out source:

    `cd google-api-go-client`
1. Fork the repo.
1. Set your fork as a remote:

    `git remote add fork git@github.com:GITHUB_USERNAME/google-api-go-client.git`
1. Make changes (see [Formatting](#formatting) and [Style](#style)), commit to
   your fork.
   
   Commit messages should follow the
   [Conventional Commit specification](https://www.conventionalcommits.org/en/v1.0.0/).
   For example:
   ```
   functions: add gophers codelab
   ```
1. Send a pull request with your changes.
1. A maintainer will review the pull request and make comments.

   Prefer adding additional commits over ammending and force-pushing since it can
   be difficult to follow code reviews when the commit history changes.

   Commits will be squashed when they're merged.

## Formatting

All code must be formatted with `gofmt` (with the latest Go version) and pass
`go vet`.

## Style

Please read and follow https://github.com/golang/go/wiki/CodeReviewComments for
all Go code in this repo.

## Contributor License Agreements

Before we can accept your pull requests you'll need to sign a Contributor
License Agreement (CLA):

- **If you are an individual writing original source code** and **you own the
intellectual property**, then you'll need to sign an [individual CLA][indvcla].
- **If you work for a company that wants to allow you to contribute your
work**, then you'll need to sign a [corporate CLA][corpcla].

You can sign these electronically (just scroll to the bottom). After that,
we'll be able to accept your pull requests.

## Contributor Code of Conduct

As contributors and maintainers of this project,
and in the interest of fostering an open and welcoming community,
we pledge to respect all people who contribute through reporting issues,
posting feature requests, updating documentation,
submitting pull requests or patches, and other activities.

We are committed to making participation in this project
a harassment-free experience for everyone,
regardless of level of experience, gender, gender identity and expression,
sexual orientation, disability, personal appearance,
body size, race, ethnicity, age, religion, or nationality.

Examples of unacceptable behavior by participants include:

* The use of sexualized language or imagery
* Personal attacks
* Trolling or insulting/derogatory comments
* Public or private harassment
* Publishing other's private information,
such as physical or electronic
addresses, without explicit permission
* Other unethical or unprofessional conduct.

Project maintainers have the right and responsibility to remove, edit, or reject
comments, commits, code, wiki edits, issues, and other contributions
that are not aligned to this Code of Conduct.
By adopting this Code of Conduct,
project maintainers commit themselves to fairly and consistently
applying these principles to every aspect of managing this project.
Project maintainers who do not follow or enforce the Code of Conduct
may be permanently removed from the project team.

This code of conduct applies both within project spaces and in public spaces
when an individual is representing the project or its community.

Instances of abusive, harassing, or otherwise unacceptable behavior
may be reported by opening an issue
or contacting one or more of the project maintainers.

This Code of Conduct is adapted from the [Contributor Covenant](http://contributor-covenant.org), version 1.2.0,
available at [http://contributor-covenant.org/version/1/2/0/](http://contributor-covenant.org/version/1/2/0/)

[indvcla]: https://developers.google.com/open-source/cla/individual
[corpcla]: https://developers.google.com/open-source/cla/corporate
