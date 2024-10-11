<!--
Guiding Principles:
Changelogs are for humans, not machines.
There should be an entry for every single version.
The same types of changes should be grouped.
Versions and sections should be linkable.
The latest version comes first.
The release date of each version is displayed.
Mention whether you follow Semantic Versioning.
Usage:
Change log entries are to be added to the Unreleased section under the
appropriate stanza (see below). Each entry should ideally include a tag and
the Github issue reference in the following format:
* (<tag>) [#<issue-number>] Changelog message.
Types of changes (Stanzas):
"Features" for new features.
"Improvements" for changes in existing functionality.
"Deprecated" for soon-to-be removed features.
"Bug Fixes" for any bug fixes.
"API Breaking" for breaking exported APIs used by developers building on SDK.
Ref: https://keepachangelog.com/en/1.0.0/
-->

# Changelog

## [Unreleased]

### Features

* [#20087](https://github.com/cosmos/cosmos-sdk/pull/20087) add `MaxVoteOptionsLen`
* [#19592](https://github.com/cosmos/cosmos-sdk/pull/19592) Add custom tally function.
* [#19304](https://github.com/cosmos/cosmos-sdk/pull/19304) Add `MsgSudoExec` for allowing executing any message as a sudo.
* [#19101](https://github.com/cosmos/cosmos-sdk/pull/19101) Add message based params configuration.
* [#18532](https://github.com/cosmos/cosmos-sdk/pull/18532) Add SPAM vote to proposals.
* [#18532](https://github.com/cosmos/cosmos-sdk/pull/18532) Add proposal types to proposals.
* [#18620](https://github.com/cosmos/cosmos-sdk/pull/18620) Add optimistic proposals.
* [#18762](https://github.com/cosmos/cosmos-sdk/pull/18762) Add multiple choice proposals.

### Improvements

* [#20521](https://github.com/cosmos/cosmos-sdk/pull/20521) Legacy proposals can now access the `appmodule.Environment` present in the `context.Context` of the handler. This is useful when migrating to server/v2 and removing the sdk context dependency.
* [#19741](https://github.com/cosmos/cosmos-sdk/pull/19741) Add `ExpeditedQuorum` parameter specifying a minimum quorum for expedited proposals, that can differ from the regular quorum.
* [#19352](https://github.com/cosmos/cosmos-sdk/pull/19352) `TallyResult` include vote options counts. Those counts replicates the now deprecated (but not removed) yes, no, abstain and veto count fields.
* [#18976](https://github.com/cosmos/cosmos-sdk/pull/18976) Log and send an event when a proposal deposit refund or burn has failed.
* [#18856](https://github.com/cosmos/cosmos-sdk/pull/18856) Add `ProposalCancelMaxPeriod` parameter for modifying how long a proposal can be cancelled after it has been submitted.
* [#19167](https://github.com/cosmos/cosmos-sdk/pull/19167) Add `YesQuorum` parameter specifying a minimum of yes vote in the total proposal voting power for the proposal to pass.
* [#18445](https://github.com/cosmos/cosmos-sdk/pull/18445) Extend gov config.
* [#18532](https://github.com/cosmos/cosmos-sdk/pull/18532) Repurpose `govcliutils.NormalizeProposalType` to work for gov v1 proposal types.

### State Machine Breaking

* [#19101](https://github.com/cosmos/cosmos-sdk/pull/19101) Add message based params configuration.
* [#18532](https://github.com/cosmos/cosmos-sdk/pull/18532) Add SPAM vote to proposals.
* [#18532](https://github.com/cosmos/cosmos-sdk/pull/18532) Add proposal types to proposals.
* [#18620](https://github.com/cosmos/cosmos-sdk/pull/18620) Add optimistic proposals.
* [#18762](https://github.com/cosmos/cosmos-sdk/pull/18762) Add multiple choice proposals.
* [#18856](https://github.com/cosmos/cosmos-sdk/pull/18856) Add `ProposalCancelMaxPeriod` parameters.
* [#19167](https://github.com/cosmos/cosmos-sdk/pull/19167) Add `YesQuorum` parameter.
* [#20348](https://github.com/cosmos/cosmos-sdk/pull/20348) Limit gov execution of proposals to a max gas limit. The limit was added to parameters and can be modified. With this version the default is set to 10 million gas. Before it was infinite gas.

### Client Breaking Changes

* [#19101](https://github.com/cosmos/cosmos-sdk/pull/19101) Querying specific params types was deprecated in gov/v1 and has been removed. gov/v1beta1 rest unchanged.

### API Breaking Changes

* [#19850](https://github.com/cosmos/cosmos-sdk/pull/19850) Removes the use of Accounts String method: 
    * `NewDeposit`, `NewMsgDeposit`, `NewMsgVote`, `NewMsgVoteWeighted`, `NewVote`, `NewProposal`, `NewMsgSubmitProposal` now take a string as an argument instead of an `sdk.AccAddress`.
    * `Prompt` and `PromptMetadata` take an address.Codec as arguments.
    * `SetProposer` takes a String as an argument instead of a `fmt.Stringer`.
* [#19481](https://github.com/cosmos/cosmos-sdk/pull/19481) Migrate module to use `appmodule.Environment`; `NewKeeper` now takes `appmodule.Environment` instead of a store service and no `baseapp.MessageRouter` anymore.
* [#19481](https://github.com/cosmos/cosmos-sdk/pull/19481) v1beta1 proposal handlers now take a `context.Context` instead of an `sdk.Context`.
* [#19592](https://github.com/cosmos/cosmos-sdk/pull/19592) `types.Config` and `types.DefaultConfig` have been moved to the keeper package in order to support the custom tallying function.
* [#19349](https://github.com/cosmos/cosmos-sdk/pull/19349) Simplify state management in `x/gov`. Note `k.VotingPeriodProposals` and `k.SetProposal` are no longer needed and have been removed.
* [#18532](https://github.com/cosmos/cosmos-sdk/pull/18532) All functions that were taking an expedited bool parameter now take a `ProposalType` parameter instead.
* [#17496](https://github.com/cosmos/cosmos-sdk/pull/17496) in `x/gov/types/v1beta1/vote.go` `NewVote` was removed, constructing the struct is required for this type.
* [#19101](https://github.com/cosmos/cosmos-sdk/pull/19101) Move `QueryProposalVotesParams` and `QueryVoteParams` from the `types/v1` package to `utils` and remove unused `querier.go` file.
* [#19740](https://github.com/cosmos/cosmos-sdk/pull/19740) `InitGenesis` and `ExportGenesis` module code and keeper code do not panic but return errors.

### Deprecated

* [#18532](https://github.com/cosmos/cosmos-sdk/pull/18532) The field `v1.Proposal.Expedited` is deprecated and will be removed in the next release.
