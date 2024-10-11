---
name: Module Readiness Checklist
about: Pre-flight checklist that modules must pass in order to be included in a release of the Cosmos SDK
labels: 'module-readiness-checklist'
---

## x/{MODULE_NAME} Module Readiness Checklist

This checklist is to be used for tracking the final internal audit of new Cosmos SDK modules prior to inclusion in a published release.

### Release Candidate Checklist

The following checklist should be gone through once the module has been fully implemented. This audit should be performed directly on `main`, or preferably on a `alpha` or `beta` release tag that includes the module.

The module **should not** be included in any Release Candidate tag until it has passed this checklist.

- [ ] API audit (at least 1 person) (@assignee)
  - [ ] Are Msg and Query methods and types well-named and organized?
  - [ ] Is everything well documented (inline godoc as well as the spec [README.md](https://github.com/cosmos/cosmos-sdk/blob/main/docs/spec/README.md) in module directory)
- [ ] State machine audit (at least 2 people) (@assignee1, @assignee2)
  - [ ] Read through MsgServer code and verify correctness upon visual inspection
  - [ ] Ensure all state machine code which could be confusing is properly commented
  - [ ] Make sure state machine logic matches Msg method documentation
  - [ ] Ensure that all state machine edge cases are covered with tests and that test coverage is sufficient (at least 90% coverage on module code)
  - [ ] Assess potential threats for each method including spam attacks and ensure that threats have been addressed sufficiently. This should be done by writing up threat assessment for each method
  - [ ] Assess potential risks of any new third party dependencies and decide whether a dependency audit is needed
- [ ] Completeness audit, fully implemented with tests (at least 1 person) (@assignee)
  - [ ] Genesis import and export of all state
  - [ ] Query services
  - [ ] CLI methods
  - [ ] All necessary migration scripts are present (if this is an upgrade of existing module)

### Published Release Checklist

After the above checks have been audited and the module is included in a tagged Release Candidate, the following additional checklist should be undertaken for live testing, and potentially a 3rd party audit (if deemed necessary):

- [ ] Testnet / devnet testing (2-3 people) (@assignee1, @assignee2, @assignee3)
  - [ ] All Msg methods have been tested especially in light of any potential threats identified
  - [ ] Genesis import and export has been tested
- [ ] Nice to have (and needed in some cases if threats could be high): Official 3rd party audit
