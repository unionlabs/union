# How to contribute

Here are some ways you can contribute:

- Give this library a star on GitHub.  It doesn't cost anything and it lets maintainers know you appreciate their work.
- Use this library in your project.  By using this library, you're more likely to open an issue with feature request, etc.
- Report security vulnerabilities privately by email after reading this contributing guide and [Security Policy](https://github.com/fxamacker/cbor#security-policy).
- Open an issue with a feature request.  It can help prioritize issues if you provide a link to your project and mention if a missing feature prevents your project from using this library.
- Open an issue with a bug report.  It's helpful if the bug report includes a link to a reproducer at [Go Playground](https://go.dev/play/).
- Open a PR that would close a specific issue.  Ask if it's a good time to open a PR in the issue because a solution might already be in progress.  Please also read about the signing requirements before spending time on a PR.

If you'd like to contribute code or send CBOR data, please read on (it can save you time!)

## Private reports

Usually, all issues are tracked publicly on [GitHub](https://github.com/fxamacker/cbor/issues). 

To report security vulnerabilities, please email faye.github@gmail.com and allow time for the problem to be resolved before disclosing it to the public.  For more info, see [Security Policy](https://github.com/fxamacker/cbor#security-policy).

Please do not send data that might contain personally identifiable information, even if you think you have permission.  That type of support requires payment and a contract where I'm indemnified, held harmless, and defended for any data you send to me.

## Pull requests

Pull requests have signing requirements and must not be anonymous.  Exceptions can be made for docs and CI scripts.

See our [Pull Request Template](https://github.com/fxamacker/cbor/blob/master/.github/pull_request_template.md) for details.

Please [create an issue](https://github.com/fxamacker/cbor/issues/new/choose), if one doesn't already exist, and describe your concern. You'll need a [GitHub account](https://github.com/signup/free) to do this.

If you submit a pull request without creating an issue and getting a response, you risk having your work unused because the bugfix or feature was already done by others and being reviewed before reaching Github.

## Describe your issue

Clearly describe the issue:
* If it's a bug, please provide: **version of this library** and **Go** (`go version`), **unmodified error message**, and describe **how to reproduce it**.  Also state **what you expected to happen** instead of the error.
* If you propose a change or addition, try to give an example how the improved code could look like or how to use it.
* If you found a compilation error, please confirm you're using a supported version of Go. If you are, then provide the output of `go version` first, followed by the complete error message.

## Please don't

Please don't send data containing personally identifiable information, even if you think you have permission.  That type of support requires payment and a contract where I'm indemnified, held harmless, and defended for any data you send to me.

Please don't send CBOR data larger than 512 bytes. If you want to send crash-producing CBOR data > 512 bytes, please get my permission before sending it to me.

## Wanted

* Opening issues that are helpful to the project
* Using this library in your project and letting me know
* Sending well-formed CBOR data (<= 512 bytes) that causes crashes (none found yet).
* Sending malformed CBOR data (<= 512 bytes) that causes crashes (none found yet, but bad actors are better than me at breaking things).
* Sending tests or data for unit tests that increase code coverage (currently around 98%)
* Pull requests with small changes that are well-documented and easily understandable.
* Sponsors, donations, bounties, or subscriptions.

## Credits

- This guide used nlohmann/json contribution guidelines for inspiration as suggested in issue #22.
- Special thanks to @lukseven for pointing out the contribution guidelines didn't mention signing requirements.
