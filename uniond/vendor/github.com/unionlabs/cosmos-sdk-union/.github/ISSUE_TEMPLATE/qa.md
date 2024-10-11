<!-- < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < < ☺ 
v                            ✰  Thanks for opening an issue! ✰    
v    Before smashing the submit button please review the template.
v    Word of caution: poorly thought-out proposals may be rejected 
v                     without deliberation 
☺ > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > >  -->

## Summary

<!-- In a few short sentences summarize the release -->

## Major Changes

<!-- Describe the major changes associated to this release -->

## Gotchas

<!-- Gotchas is an area which changes could of been made that the auditors should be aware of -->

## QA Breakdown

* Audit
    * [ ] Audit BaseApp
    * [ ] Audit Types
    * [ ] Audit x/auth 
    * [ ] Audit x/authz
    * [ ] Audit x/bank
    * [ ] Audit x/bank/v2
    * [ ] Audit x/circuit
    * [ ] Audit x/consensus 
    * [ ] Audit x/crisis 
    * [ ] Audit x/distribution 
    * [ ] Audit x/evidence 
    * [ ] Audit x/epochs
    * [ ] Audit x/feegrant
    * [ ] Audit x/genutil 
    * [ ] Audit x/gov 
    * [ ] Audit x/group 
    * [ ] Audit x/mint 
    * [ ] Audit x/nft 
    * [ ] Audit x/protocolpool
    * [ ] Audit x/slashing 
    * [ ] Audit x/staking
    * [ ] Audit x/tx 
    * [ ] Audit x/upgrade 
    * [ ] Audit client
    * [ ] Audit server
    * [ ] Audit store 
    * [ ] Audit runtime
    * [ ] Audit simapp
* [ ] Release alpha
* [ ] Cosmos-SDK testnet
* [ ] Public testnet (IBC, WASM, SDK)
* [ ] Upgrade a chain with data from vX
* Release documentation
    * [ ] Audit UPGRADING.md
    * [ ] Update all codeblock to the appropriate version number


### Audit checklist

* please copy to a markdown to follow while you walk through the code
* 2 people should be assigned to each section 

* [ ] API audit 
    * spec audit: check if the spec is complete.
    * Are Msg and Query methods and types well-named and organized?
    * Is everything well documented (inline godoc as well as package [`README.md`](https://docs.cosmos.network/main/spec/SPEC_MODULE#common-layout) in module directory)
    * check the proto definition - make sure everything is in accordance to ADR-30 (at least 1 person, TODO assignee)
        * Check new fields and endpoints have the `Since: cosmos-sdk X` comment
* [ ] Completeness audit, fully implemented with tests
    * [ ] Genesis import and export of all state
    * [ ] Query services
    * [ ] CLI methods
    * [ ] All necessary migration scripts are present (if this is an upgrade of existing module)
* [ ] State machine audit
    * [ ] Read through MsgServer code and verify correctness upon visual inspection
    * [ ] Ensure all state machine code which could be confusing is properly commented
    * [ ] Make sure state machine logic matches Msg method documentation
    * [ ] Ensure that all state machine edge cases are covered with tests and that test coverage is sufficient (at least 90% coverage on module code)
    * [ ] Assess potential threats for each method including spam attacks and ensure that threats have been addressed sufficiently. This should be done by writing up threat assessment for each method. Specifically we should be paying attention to: 
        * [ ] algorithmic complexity and places this could be exploited (ex. nested `for` loops)
        * [ ] charging gas complex computation (ex. `for` loops)
        * [ ] storage is safe (we don't pollute the state).
    * [ ] Assess potential risks of any new third party dependencies and decide whether a dependency audit is needed
    * [ ] Check correctness of simulation implementation if any
* [ ] Audit Changelog against commit log, ensuring all breaking changes, bug fixes, and improvements are properly documented.

If any changes are needed, please make them against main and backport them to release/vX.X.x
