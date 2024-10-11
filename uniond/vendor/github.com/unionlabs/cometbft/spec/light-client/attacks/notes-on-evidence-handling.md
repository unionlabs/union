
# Light client attacks

We define a light client attack as detection of conflicting headers for a given height that can be verified
starting from the trusted light block. A light client attack is defined in the context of interactions of
light client with two peers. One of the peers (called primary) defines a trace of verified light blocks
(primary trace) that are being checked against trace of the other peer (called witness) that we call
witness trace.

A light client attack is defined by the primary and witness traces
that have a common root (the same trusted light block for a common height) but forms
conflicting branches (end of traces is for the same height but with different headers).
Note that conflicting branches could be arbitrarily big as branches continue to diverge after
a bifurcation point. We propose an approach that allows us to define a valid light client attack
only with a common light block and a single conflicting light block. We rely on the fact that
we assume that the primary is under suspicion (therefore not trusted) and that the witness plays
support role to detect and process an attack (therefore trusted). Therefore, once a light client
detects an attack, it needs to send to a witness only missing data (common height
and conflicting light block) as it has its trace. Keeping light client attack data of constant size
saves bandwidth and reduces an attack surface. As we will explain below, although in the context of
light client core
[verification](https://github.com/cometbft/cometbft/tree/main/spec/light-client/verification)
the roles of primary and witness are clearly defined,
in case of the attack, we run the same attack detection procedure twice where the roles are swapped.
The rationale is that the light client does not know what peer is correct (on a right main branch)
so it tries to create and submit an attack evidence to both peers.

Light client attack evidence consists of a conflicting light block and a common height.

```go
type LightClientAttackEvidence struct {
    ConflictingBlock   LightBlock
    CommonHeight       int64
}
```

Full node can validate a light client attack evidence by executing the following procedure:

```go
func IsValid(lcaEvidence LightClientAttackEvidence, bc Blockchain) boolean {
    commonBlock = GetLightBlock(bc, lcaEvidence.CommonHeight)
    if commonBlock == nil return false

    // Note that trustingPeriod in ValidAndVerified is set to UNBONDING_PERIOD
    verdict = ValidAndVerified(commonBlock, lcaEvidence.ConflictingBlock)
    conflictingHeight = lcaEvidence.ConflictingBlock.Header.Height

    return verdict == OK and bc[conflictingHeight].Header != lcaEvidence.ConflictingBlock.Header
}
```

## Light client attack creation

Given a trusted light block `trusted`, a light node executes the bisection algorithm to verify header
`untrusted` at some height `h`. If the bisection algorithm succeeds, then the header `untrusted` is verified.
Headers that are downloaded as part of the bisection algorithm are stored in a store and they are also in
the verified state. Therefore, after the bisection algorithm successfully terminates we have a trace of
the light blocks ([] LightBlock) we obtained from the primary that we call primary trace.

### Primary trace

The following invariant holds for the primary trace:

- Given a `trusted` light block, target height `h`, and `primary_trace` ([] LightBlock):
    *primary_trace[0] == trusted* and *primary_trace[len(primary_trace)-1].Height == h* and
    successive light blocks are passing light client verification logic.

### Witness with a conflicting header

The verified header at height `h` is cross-checked with every witness as part of
[detection](https://github.com/cometbft/cometbft/tree/main/spec/light-client/detection).
If a witness returns the conflicting header at the height `h` the following procedure is executed to verify
if the conflicting header comes from the valid trace and if that's the case to create an attack evidence:

#### Helper functions

We assume the following helper functions:

```go
// Returns trace of verified light blocks starting from rootHeight and ending with targetHeight.
Trace(lightStore LightStore, rootHeight int64, targetHeight int64) LightBlock[]

// Returns validator set for the given height
GetValidators(bc Blockchain, height int64) Validator[]

// Returns validator set for the given height
GetValidators(bc Blockchain, height int64) Validator[]

// Return validator addresses for the given validators
GetAddresses(vals Validator[]) ValidatorAddress[]
```

```go
func DetectLightClientAttacks(primary PeerID,
                              primary_trace []LightBlock,
                              witness PeerID) (LightClientAttackEvidence, LightClientAttackEvidence) {
    primary_lca_evidence, witness_trace = DetectLightClientAttack(primary_trace, witness)

    witness_lca_evidence = nil
    if witness_trace != nil {
        witness_lca_evidence, _ = DetectLightClientAttack(witness_trace, primary)
    }
    return primary_lca_evidence, witness_lca_evidence
}

func DetectLightClientAttack(trace []LightBlock, peer PeerID) (LightClientAttackEvidence, []LightBlock) {

    lightStore = new LightStore().Update(trace[0], StateTrusted)

    for i in 1..len(trace)-1 {
        lightStore, result = VerifyToTarget(peer, lightStore, trace[i].Header.Height)

        if result == ResultFailure then return (nil, nil)

        current = lightStore.Get(trace[i].Header.Height)

        // if obtained header is the same as in the trace we continue with a next height
        if current.Header == trace[i].Header continue

        // we have identified a conflicting header
        commonBlock = trace[i-1]
        conflictingBlock = trace[i]

        return (LightClientAttackEvidence { conflictingBlock, commonBlock.Header.Height },
                Trace(lightStore, trace[i-1].Header.Height, trace[i].Header.Height))
    }
    return (nil, nil)
}
```

## Evidence handling

As part of on chain evidence handling, full nodes identifies misbehaving processes and informs
the application, so they can be slashed. Note that only bonded validators should
be reported to the application. There are three types of attacks that can be executed against
light client:

- lunatic attack
- equivocation attack and
- amnesia attack.  

We now specify the evidence handling logic.

```go
func detectMisbehavingProcesses(lcAttackEvidence LightClientAttackEvidence, bc Blockchain) []ValidatorAddress {
   assume IsValid(lcaEvidence, bc)

   // lunatic light client attack
   if !isValidBlock(current.Header, conflictingBlock.Header) {
        conflictingCommit = lcAttackEvidence.ConflictingBlock.Commit
        bondedValidators = GetNextValidators(bc, lcAttackEvidence.CommonHeight)

        return getSigners(conflictingCommit) intersection GetAddresses(bondedValidators)

   // equivocation light client attack
   } else if current.Header.Round == conflictingBlock.Header.Round {
        conflictingCommit = lcAttackEvidence.ConflictingBlock.Commit
        trustedCommit = bc[conflictingBlock.Header.Height+1].LastCommit

        return getSigners(trustedCommit) intersection getSigners(conflictingCommit)

   // amnesia light client attack
   } else {
        HandleAmnesiaAttackEvidence(lcAttackEvidence, bc)
   }
}

// Block validity in this context is defined by the trusted header.
func isValidBlock(trusted Header, conflicting Header) boolean {
    return trusted.ValidatorsHash == conflicting.ValidatorsHash and
           trusted.NextValidatorsHash == conflicting.NextValidatorsHash and
           trusted.ConsensusHash == conflicting.ConsensusHash and
           trusted.AppHash == conflicting.AppHash and
           trusted.LastResultsHash == conflicting.LastResultsHash
}

func getSigners(commit Commit) []ValidatorAddress {
    signers = []ValidatorAddress
    for (i, commitSig) in commit.Signatures {
        if commitSig.BlockIDFlag == BlockIDFlagCommit {
            signers.append(commitSig.ValidatorAddress)
        }
    }
    return signers
}
```

Note that amnesia attack evidence handling involves more complex processing, i.e., cannot be
defined simply on amnesia attack evidence. We explain in the following section a protocol
for handling amnesia attack evidence.

### Amnesia attack evidence handling

Detecting faulty processes in case of the amnesia attack is more complex and cannot be inferred
purely based on attack evidence data. In this case, in order to detect misbehaving processes we need
access to votes processes sent/received during the conflicting height. Therefore, amnesia handling assumes that
validators persist all votes received and sent during multi-round heights (as amnesia attack
is only possible in heights that executes over multiple rounds, i.e., commit round > 0).  

To simplify description of the algorithm we assume existence of the trusted oracle called monitor that will
drive the algorithm and output faulty processes at the end. Monitor can be implemented in a
distributed setting as on-chain module. The algorithm works as follows:
    1) Monitor sends votesets request to validators of the conflicting height. Validators
    are expected to send their votesets within predefined timeout.
    2) Upon receiving votesets request, validators send their votesets to a monitor.  
    2) Validators which have not sent its votesets within timeout are considered faulty.
    3) The preprocessing of the votesets is done. That means that the received votesets are analyzed
    and each vote (valid) sent by process p is added to the voteset of the sender p. This phase ensures that
    votes sent by faulty processes observed by at least one correct validator cannot be excluded from the analysis.
    4) Votesets of every validator are analyzed independently to decide whether the validator is correct or faulty.
       A faulty validators is the one where at least one of those invalid transitions is found:
            - More than one PREVOTE message is sent in a round
            - More than one PRECOMMIT message is sent in a round
            - PRECOMMIT message is sent without receiving +2/3 of voting-power equivalent
            appropriate PREVOTE messages
            - PREVOTE message is sent for the value V’ in round r’ and the PRECOMMIT message had
            been sent for the value V in round r by the same process (r’ > r) and there are no
            +2/3 of voting-power equivalent PREVOTE(vr, V’) messages (vr ≥ 0 and vr > r and vr < r’)
            as the justification for sending PREVOTE(r’, V’)
