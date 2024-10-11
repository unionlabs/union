<!-- markdown-link-check-disable -->
# Lightclient Attackers Isolation

Adversarial nodes may have the incentive to lie to a lightclient about the
state of a Cosmos blockchain, built using Tendermint consensus algorithm.
An attempt to do so is called attack. Light client [verification][verification] checks incoming data by checking a so-called "commit", which is a forwarded set of signed messages that is (supposedly) produced during executing Tendermint consensus. Thus, an attack boils down to creating and signing Tendermint consensus messages in deviation from the Tendermint consensus algorithm rules.

As Tendermint consensus and light client verification is safe under the assumption of more than 2/3 of correct voting power per block [[CMBC-FM-2THIRDS]][CMBC-FM-2THIRDS-link], this implies that if there was an attack then [[CMBC-FM-2THIRDS]][CMBC-FM-2THIRDS-link] was violated, that is, there is a block such that

- validators deviated from the protocol, and
- these validators represent more than 1/3 of the voting power in that block.

In the case of an [attack][node-based-attack-characterization], the lightclient [attack detection mechanism][detection] computes data, so called evidence [[LC-DATA-EVIDENCE.1]][LC-DATA-EVIDENCE-link], that can be used

- to proof that there has been attack [[CMBC-LC-EVIDENCE-DATA.1]][CMBC-LC-EVIDENCE-DATA-link] and
- as basis to find the actual nodes that deviated from the Tendermint algorithm.

This specification considers how a full node in a Cosmos blockchain can isolate a set of attackers that launched the attack. The set should satisfy

- the set does not contain a correct validator
- the set contains validators that represent more than 1/3 of the voting power of a block that is still within the unbonding period

# Outline

After providing the [problem statement](#Part-I---Basics-and-Definition-of-the-Problem), we specify the [isolator function](#Part-II---Protocol) and close with the discussion about its [correctness](#Part-III---Completeness) which is based on computer-aided analysis of Tendermint consensus algorithm.

# Part I - Basics and Definition of the Problem

For definitions of data structures used here, in particular LightBlocks [[LCV-DATA-LIGHTBLOCK.1]](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_002_draft.md#lcv-data-lightblock1), we refer to the specification of [Light Client Verification][verification].

The specification of the [detection mechanism][detection] describes

- what is a light client attack,
- conditions under which the detector will detect a light client attack,
- and the format of the output data, called evidence, in the case an attack is detected. The format is defined in
[[LC-DATA-EVIDENCE.1]][LC-DATA-EVIDENCE-link] and looks as follows

```go
type LightClientAttackEvidence struct {
    ConflictingBlock   LightBlock
    CommonHeight       int64
}
```

The isolator is a function that gets as input evidence `ev`
and a prefix of the blockchain `bc` at least up to height `ev.ConflictingBlock.Header.Height + 1`. The output is a set of *peerIDs* of validators.

We assume that the full node is synchronized with the blockchain and has reached the height `ev.ConflictingBlock.Header.Height + 1`.

#### **[LCAI-INV-Output.1]**

When an output is generated it satisfies the following properties:

- If
    - `bc[CommonHeight].bfttime` is within the unbonding period w.r.t. the time at the full node,
    - `ev.ConflictingBlock.Header != bc[ev.ConflictingBlock.Header.Height]`
    - Validators in `ev.ConflictingBlock.Commit` represent more than 1/3 of the voting power in `bc[ev.CommonHeight].NextValidators`
- Then: The output is a set of validators in `bc[CommonHeight].NextValidators` that
    - represent more than 1/3 of the voting power in `bc[ev.commonHeight].NextValidators`
    - signed Tendermint consensus messages for height `ev.ConflictingBlock.Header.Height` by violating the Tendermint consensus algorithm.
- Else: the empty set.

# Part II - Protocol

Here we discuss how to solve the problem of isolating misbehaving processes. We describe the function `isolateMisbehavingProcesses` as well as all the helping functions below. In [Part III](#part-III---Completeness), we discuss why the solution is complete based on result from analysis with automated tools.

## Isolation

### Outline

We first check whether the conflicting block can indeed be verified from the common height. We then first check whether it was a lunatic attack (violating validity). If this is not the case, we check for equivocation. If this also is not the case, we start the on-chain <!-- markdown-link-check-disable-next-line -->[accountability protocol](https://docs.google.com/document/d/11ZhMsCj3y7zIZz4udO9l25xqb0kl7gmWqNpGVRzOeyY/edit).

#### **[LCAI-FUNC-MAIN.1]**

```go
func isolateMisbehavingProcesses(ev LightClientAttackEvidence, bc Blockchain) []ValidatorAddress {

    reference := bc[ev.conflictingBlock.Header.Height].Header
    ev_header := ev.conflictingBlock.Header

    ref_commit := bc[ev.conflictingBlock.Header.Height + 1].Header.LastCommit // + 1 !!
    ev_commit := ev.conflictingBlock.Commit

    if violatesTMValidity(reference, ev_header) {
        // lunatic light client attack
        signatories := Signers(ev.ConflictingBlock.Commit)
        bonded_vals := Addresses(bc[ev.CommonHeight].NextValidators)
        return intersection(signatories,bonded_vals)

    }
    // If this point is reached the validator sets in reference and ev_header are identical
    else if RoundOf(ref_commit) == RoundOf(ev_commit) {
        // equivocation light client attack
        return intersection(Signers(ref_commit), Signers(ev_commit))
    }
    else {
        // amnesia light client attack
        return IsolateAmnesiaAttacker(ev, bc)
    }
}
```

- Implementation comment
    - If the full node has only reached height `ev.conflictingBlock.Header.Height` then `bc[ev.conflictingBlock.Header.Height + 1].Header.LastCommit` refers to the locally stored commit for this height. (This commit must be present by the precondition on `length(bc)`.)
    - We check in the precondition that the unbonding period is not expired. However, since time moves on, before handing the validators over Cosmos SDK, the time needs to be checked again to satisfy the contract which requires that only bonded validators are reported. This passing of validators to the SDK is out of scope of this specification.
- Expected precondition
    - `length(bc) >= ev.conflictingBlock.Header.Height`
    - `ValidAndVerifiedUnbonding(bc[ev.CommonHeight], ev.ConflictingBlock) == SUCCESS`
    - `ev.ConflictingBlock.Header != bc[ev.ConflictingBlock.Header.Height]`
    - `ev.conflictingBlock` satisfies basic validation (in particular all signed messages in the Commit are from the same round)
- Expected postcondition
    - [[FN-INV-Output.1]](#FN-INV-Output1) holds
- Error condition
    - returns an error if precondition is violated.

### Details of the Functions

#### **[LCAI-FUNC-VVU.1]**

```go
func ValidAndVerifiedUnbonding(trusted LightBlock, untrusted LightBlock) Result
```

- Conditions are identical to [[LCV-FUNC-VALID.2]][LCV-FUNC-VALID.link] except the precondition "*trusted.Header.Time > now - trustingPeriod*" is substituted with
    - `trusted.Header.Time > now - UnbondingPeriod`

#### **[LCAI-FUNC-NONVALID.1]**

```go
func violatesTMValidity(ref Header, ev Header) boolean
```

- Implementation remarks
    - checks whether the evidence header `ev` violates the validity property of Tendermint consensus algorithm, by checking against a reference header
- Expected precondition
    - `ref.Height == ev.Height`
- Expected postcondition
    - returns evaluation of the following disjunction  
    **[LCAI-NONVALID-OUTPUT.1]** ==  
    `ref.ValidatorsHash != ev.ValidatorsHash` or  
    `ref.NextValidatorsHash != ev.NextValidatorsHash` or  
    `ref.ConsensusHash != ev.ConsensusHash` or  
    `ref.AppHash != ev.AppHash` or  
    `ref.LastResultsHash != ev.LastResultsHash`

```go
func IsolateAmnesiaAttacker(ev LightClientAttackEvidence, bc Blockchain) []ValidatorAddress
```

- Implementation remarks <!-- markdown-link-check-disable-next-line -->
    - This triggers the [query/response protocol](https://docs.google.com/document/d/11ZhMsCj3y7zIZz4udO9l25xqb0kl7gmWqNpGVRzOeyY/edit).
- Expected postcondition
    - returns attackers according to [LCAI-INV-Output.1].

```go
func RoundOf(commit Commit) []ValidatorAddress
```

- Expected precondition
    - `commit` is well-formed. In particular all votes are from the same round `r`.
- Expected postcondition
    - returns round `r` that is encoded in all the votes of the commit
- Error condition
    - reports error if precondition is violated

```go
func Signers(commit Commit) []ValidatorAddress
```

- Expected postcondition
    - returns all validator addresses in `commit`

```go
func Addresses(vals Validator[]) ValidatorAddress[]
```

- Expected postcondition
    - returns all validator addresses in `vals`

# Part III - Completeness

As discussed in the beginning of this document, an attack boils down to creating and signing Tendermint consensus messages in deviation from the Tendermint consensus algorithm rules.
The main function `isolateMisbehavingProcesses` distinguishes three kinds of wrongly signed messages, namely,

- lunatic: signing invalid blocks
- equivocation: double-signing valid blocks in the same consensus round
- amnesia: signing conflicting blocks in different consensus rounds, without having seen a quorum of messages that would have allowed to do so.

The question is whether this captures all attacks.
First observe that the first check in `isolateMisbehavingProcesses` is `violatesTMValidity`. It takes care of lunatic attacks. If this check passes, that is, if `violatesTMValidity` returns `FALSE` this means that [[LCAI-NONVALID-OUTPUT.1]](#LCAI-FUNC-NONVALID1]) evaluates to false, which implies that `ref.ValidatorsHash = ev.ValidatorsHash`. Hence, after `violatesTMValidity`, all the involved validators are the ones from the blockchain. It is thus sufficient to analyze one instance of Tendermint consensus with a fixed group membership (set of validators). Also, as we have two different blocks for the same height, it is sufficient to consider two different valid consensus values, that is, binary consensus.

For this fixed group membership, we have analyzed the attacks using the TLA+ specification of [Tendermint Consensus in TLA+][tendermint-accountability]. We checked that indeed the only possible scenarios that can lead to violation of agreement are **equivocation** and **amnesia**. An independent study by Galois of the protocol based on [Ivy proofs](https://github.com/cometbft/cometbft/tree/main/spec/ivy-proofs) led to the same conclusion.

# References

[[supervisor]] The specification of the light client supervisor.

[[verification]] The specification of the light client verification protocol.

[[detection]] The specification of the light client attack detection mechanism.


[tendermint-accountability]:
https://github.com/cometbft/cometbft/tree/main/spec/light-client/accountability

[supervisor]:
https://github.com/cometbft/cometbft/blob/main/spec/light-client/supervisor/supervisor_001_draft.md

[verification]: https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_002_draft.md

[detection]:
https://github.com/cometbft/cometbft/blob/main/spec/light-client/detection/detection_003_reviewed.md

[LC-DATA-EVIDENCE-link]:
https://github.com/cometbft/cometbft/blob/main/spec/light-client/detection/detection_003_reviewed.md#lc-data-evidence1

[CMBC-LC-EVIDENCE-DATA-link]:
https://github.com/cometbft/cometbft/blob/main/spec/light-client/detection/detection_003_reviewed.md#cmbc-lc-evidence-data1

[node-based-attack-characterization]:
https://github.com/cometbft/cometbft/blob/main/spec/light-client/detection/detection_003_reviewed.md#block-based-characterization-of-attacks

[CMBC-FM-2THIRDS-link]: https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_002_draft.md#cmbc-fm-2thirds1

[LCV-FUNC-VALID.link]: https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_002_draft.md#lcv-func-valid2
