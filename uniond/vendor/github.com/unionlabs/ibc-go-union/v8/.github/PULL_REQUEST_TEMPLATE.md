<!-- < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < ☺
v                               ✰  Thanks for creating a PR! ✰
v    Before smashing the submit button please review the checkboxes.
v    If a checkbox is n/a - please still include it but + a little note why
☺ > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > >  -->

## Description

<!-- Add a description of the changes that this PR introduces and the files that
are the most critical to review.
-->

closes: #XXXX


### Commit Message / Changelog Entry

```text
type: commit message
```

see the [guidelines](https://github.com/cosmos/ibc-go/blob/main/docs/dev/pull-requests.md#commit-messages) for commit messages. (view raw markdown for examples)


<!--
Example commit messages:

fix: skip emission of unpopulated memo field in ics20
deps: updating sdk to v0.46.4
chore: removed unused variables
e2e: adding e2e upgrade test for ibc-go/v6
docs: ics27 v6 documentation updates
feat: add semantic version utilities for e2e tests
feat(api)!: this is an api breaking feature
fix(statemachine)!: this is a statemachine breaking fix
-->

---

Before we can merge this PR, please make sure that all the following items have been
checked off. If any of the checklist items are not applicable, please leave them but
write a little note why.

- [ ] Targeted PR against correct branch (see [CONTRIBUTING.md](https://github.com/cosmos/ibc-go/blob/main/docs/dev/pull-requests.md#pull-request-targeting)).
- [ ] Linked to Github issue with discussion and accepted design OR link to spec that describes this work.
- [ ] Code follows the [module structure standards](https://github.com/cosmos/cosmos-sdk/blob/main/docs/build/building-modules/11-structure.md) and [Go style guide](../docs/dev/go-style-guide.md).
- [ ] Wrote unit and integration [tests](https://github.com/cosmos/ibc-go/blob/main/testing/README.md#ibc-testing-package).
- [ ] Updated relevant documentation (`docs/`) or specification (`x/<module>/spec/`).
- [ ] Added relevant `godoc` [comments](https://blog.golang.org/godoc-documenting-go-code).
- [ ] Provide a [commit message](https://github.com/cosmos/ibc-go/blob/main/docs/dev/pull-requests.md#commit-messages) to be used for the changelog entry in the PR description for review.
- [ ] Re-reviewed `Files changed` in the Github PR explorer.
- [ ] Review `Codecov Report` in the comment section below once CI passes.
