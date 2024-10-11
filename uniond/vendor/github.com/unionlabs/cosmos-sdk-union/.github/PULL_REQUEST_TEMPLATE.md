# Description

Closes: #XXXX

<!-- Add a description of the changes that this PR introduces and the files that
are the most critical to review. -->

---

## Author Checklist

*All items are required. Please add a note to the item if the item is not applicable and
please add links to any relevant follow up issues.*

I have...

* [ ] included the correct [type prefix](https://github.com/commitizen/conventional-commit-types/blob/v3.0.0/index.json) in the PR title, you can find examples of the prefixes below:
    <!-- * `feat`: A new feature
    * `fix`: A bug fix
    * `docs`: Documentation only changes
    * `style`: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
    * `refactor`: A code change that neither fixes a bug nor adds a feature
    * `perf`: A code change that improves performance
    * `test`: Adding missing tests or correcting existing tests
    * `build`: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
    * `ci`: Changes to our CI configuration files and scripts (example scopes: Travis, Circle, BrowserStack, SauceLabs)
    * `chore`: Other changes that don't modify src or test files
    * `revert`: Reverts a previous commit -->
* [ ] confirmed `!` in the type prefix if API or client breaking change
* [ ] targeted the correct branch (see [PR Targeting](https://github.com/cosmos/cosmos-sdk/blob/main/CONTRIBUTING.md#pr-targeting))
* [ ] provided a link to the relevant issue or specification
* [ ] reviewed "Files changed" and left comments if necessary
* [ ] included the necessary unit and integration [tests](https://github.com/cosmos/cosmos-sdk/blob/main/CONTRIBUTING.md#testing)
* [ ] added a changelog entry to `CHANGELOG.md`
* [ ] updated the relevant documentation or specification, including comments for [documenting Go code](https://blog.golang.org/godoc)
* [ ] confirmed all CI checks have passed

## Reviewers Checklist

*All items are required. Please add a note if the item is not applicable and please add
your handle next to the items reviewed if you only reviewed selected items.*

Please see [Pull Request Reviewer section in the contributing guide](../CONTRIBUTING.md#reviewer) for more information on how to review a pull request.

I have...

* [ ] confirmed the correct [type prefix](https://github.com/commitizen/conventional-commit-types/blob/v3.0.0/index.json) in the PR title
* [ ] confirmed all author checklist items have been addressed
* [ ] reviewed state machine logic, API design and naming, documentation is accurate, tests and test coverage
