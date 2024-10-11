---
name: Release tracker
about: Create an issue to track release progress
---

<!-- < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < ☺
v                            ✰  Thanks for opening an issue! ✰
v    Before smashing the submit button please review the template.
v    Word of caution: poorly thought-out proposals may be rejected
v                     without deliberation
☺ > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > >  -->

## Milestones

<!-- Links to alpha, beta, RC or final milestones -->

## IBC spec compatibility

<!-- Version of the IBC spec that this release is compatible with -->

## QA

### Backwards compatibility

<!-- List of tests that need be performed with previous
versions of ibc-go to guarantee that no regression is introduced -->

- [ ] [Compatibility tests](https://github.com/cosmos/ibc-go/actions/workflows/e2e-compatibility.yaml) pass for the release branch.
- [ ] [Upgrade tests](https://github.com/cosmos/ibc-go/actions/workflows/e2e-upgrade.yaml) pass.
- [ ] [Interchain Accounts inter-tx tests](https://github.com/cosmos/interchain-accounts-demo/actions/workflows/e2e-compatibility.yaml) pass.

### Other testing

## Migration

<!-- Link to migration document -->

## Checklist

<!-- Remove any items that are not applicable. -->

- [ ] Bump [go package version](https://github.com/cosmos/ibc-go/blob/main/go.mod#L3).
- [ ] Change all imports starting with `github.com/cosmos/ibc-go/v{x}` to `github.com/cosmos/ibc-go/v{x+1}`.
- [ ] Branch off main to create release branch in the form of `release/vx.y.z` and add branch protection rules.
- [ ] Add branch protection rules to new release branch.
- [ ] Add backport task to [`mergify.yml`](https://github.com/cosmos/ibc-go/blob/main/.github/mergify.yml)
- [ ] Upgrade ibc-go version in [ibctest](https://github.com/strangelove-ventures/ibctest).
- [ ] Check Swagger is up-to-date.

## Post-release checklist

- [ ] Update [`CHANGELOG.md`](https://github.com/cosmos/ibc-go/blob/main/CHANGELOG.md)
- [ ] Update the table of supported release lines (and End of Life dates) in [`RELEASES.md`](https://github.com/cosmos/ibc-go/blob/main/RELEASES.md):
  - Add the new release line.
  - Remove any release lines that might have become discontinued.
- [ ] Update [version matrix](https://github.com/cosmos/ibc-go/blob/main/RELEASES.md#version-matrix) in `RELEASES.md`:
  - Add the new release.
  - Remove any tags that might not be recommended anymore.
- [ ] Update the list of [supported release lines in README.md](https://github.com/cosmos/ibc-go#releases), if necessary.
- [ ] Update docs site:
  - [ ] If the release is occurring on the main branch, on the latest version, then run `npm run docusaurus docs:version vX.Y.Z` in the `docs/` directory. (where `X.Y.Z` is the new version number)
  - [ ] If the release is occurring on an older release branch, then make a PR to the main branch called `docs: new release vX.Y.Z` doing the following:
    - [ ] Update the content of the docs found in `docs/versioned_docs/version-vx.y.z` if needed. (where `x.y.z` is the previous version number)
    - [ ] Update the version number of the older release branch by changing the version number of the older release branch in:
      - [ ] In `docs/versions.json`.
      - [ ] Rename `docs/versioned_sidebars/version-vx.y.z-sidebars.json`
      - [ ] Rename `docs/versioned_docs/version-vx.y.z`
- [ ] Bump ibc-go version in [cosmos/interchain-accounts-demo repository](https://github.com/cosmos/interchain-accounts-demo) and create a tag.
- [ ] Update the [compatibility test matrices](https://github.com/cosmos/ibc-go/tree/main/.github/compatibility-test-matrices):
  - Add the new release.
  - Remove any tags that might not be recommended anymore.
- [ ] Update the manual [e2e `simd`](https://github.com/cosmos/ibc-go/blob/main/.github/workflows/e2e-manual-simd.yaml) test workflow:
  - Remove any tags that might not be recommended anymore.
- [ ] After changes to docs site are deployed, check [ibc.cosmos.network](https://ibc.cosmos.network) is updated.
- [ ] Open issue in [SDK tutorials repo](https://github.com/cosmos/sdk-tutorials) to update tutorials to the released version of ibc-go.

---

#### For Admin Use

- [ ] Not duplicate issue
- [ ] Appropriate labels applied
- [ ] Appropriate contributors tagged/assigned
