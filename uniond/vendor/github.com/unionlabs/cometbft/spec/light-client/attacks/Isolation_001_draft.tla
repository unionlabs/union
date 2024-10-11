----------------------- MODULE Isolation_001_draft ----------------------------
(**
 * The specification of the attackers isolation at full node,
 * when it has received an evidence from the light client.
 * We check that the isolation spec produces a set of validators
 * that have more than 1/3 of the voting power.
 *
 * It follows the English specification:
 *
 * https://github.com/cometbft/cometbft/blob/main/spec/light-client/attacks/isolate-attackers_001_draft.md
 *
 * The assumptions made in this specification:
 *
 *  - the voting power of every validator is 1
 *     (add more validators, if you need more validators)
 *
 *  - Cosmos security model is violated
 *    (there are Byzantine validators who signed a conflicting block)
 *
 * Igor Konnov, Zarko Milosevic, Josef Widder, Informal Systems, 2020
 *)


EXTENDS Integers, FiniteSets, Apalache

\* algorithm parameters
CONSTANTS
  AllNodes,
    (* a set of all nodes that can act as validators (correct and faulty) *)
  COMMON_HEIGHT,
    (* an index of the block header that two peers agree upon *)
  CONFLICT_HEIGHT,
    (* an index of the block header that two peers disagree upon *)
  TRUSTING_PERIOD,
    (* the period within which the validators are trusted *)
  FAULTY_RATIO
    (* a pair <<a, b>> that limits that ratio of faulty validator in the blockchain
       from above (exclusive). Cosmos security model prescribes 1 / 3. *)

VARIABLES  
  blockchain,           (* the chain at the full node *)
  refClock,             (* the reference clock at the full node *)
  Faulty,               (* the set of faulty validators *)
  conflictingBlock,     (* an evidence that two peers reported conflicting blocks *)
  state,                (* the state of the attack isolation machine at the full node *)
  attackers             (* the set of the identified attackers *)

vars == <<blockchain, refClock, Faulty, conflictingBlock, state>>  
 
\* instantiate the chain at the full node
ULTIMATE_HEIGHT == CONFLICT_HEIGHT + 1 
BC == INSTANCE Blockchain_003_draft

\* use the light client API
TRUSTING_HEIGHT == COMMON_HEIGHT
TARGET_HEIGHT == CONFLICT_HEIGHT

LC == INSTANCE LCVerificationApi_003_draft
    WITH localClock <- refClock, REAL_CLOCK_DRIFT <- 0, CLOCK_DRIFT <- 0

\* old-style type annotations in apalache
a <: b == a

\* [LCAI-NONVALID-OUTPUT.1::TLA.1]
ViolatesValidity(header1, header2) ==
    \/ header1.VS /= header2.VS
    \/ header1.NextVS /= header2.NextVS
    \/ header1.height /= header2.height
    \/ header1.time /= header2.time
    (* The English specification also checks the fields that we do not have
       at this level of abstraction:
       - header1.ConsensusHash != header2.ConsensusHash or
       - header1.AppHash != header2.AppHash or
       - header1.LastResultsHash header2 != ev.LastResultsHash
    *)

Init ==
    /\ state := "init"
    \* Pick an arbitrary blockchain from 1 to COMMON_HEIGHT + 1.
    /\ BC!InitToHeight(FAULTY_RATIO) \* initializes blockchain, Faulty, and refClock
    /\ attackers := {} <: {STRING}      \* attackers are unknown
    \* Receive an arbitrary evidence.
    \* Instantiate the light block fields one by one,
    \* to avoid combinatorial explosion of records.
    /\ \E time \in Int:
        \E VS, NextVS, lastCommit, Commits \in SUBSET AllNodes:
          LET conflicting ==
             [ Commits |-> Commits,
               header |->
                 [height |-> CONFLICT_HEIGHT,
                  time |-> time,
                  VS |-> VS,
                  NextVS |-> NextVS,
                  lastCommit |-> lastCommit] ]
          IN  
          LET refBlock == [ header |-> blockchain[COMMON_HEIGHT],
                           Commits |-> blockchain[COMMON_HEIGHT + 1].lastCommit ]
          IN
          /\ "SUCCESS" = LC!ValidAndVerifiedUntimed(refBlock, conflicting)
          \* More than third of next validators in the common reference block
          \* is faulty. That is a precondition for a fork.
          /\ 3 * Cardinality(Faulty \intersect refBlock.header.NextVS)
                > Cardinality(refBlock.header.NextVS)
          \* correct validators cannot sign an invalid block
          /\ ViolatesValidity(conflicting.header, refBlock.header)
              => conflicting.Commits \subseteq Faulty
          /\ conflictingBlock := conflicting

    
\* This is a specification of isolateMisbehavingProcesses.
\*
\* [LCAI-FUNC-MAIN.1::TLA.1]
Next ==
    /\ state = "init"
    \* Extract the rounds from the reference block and the conflicting block.
    \* In this specification, we just pick rounds non-deterministically.
    \* The English specification calls RoundOf on the blocks.
    /\ \E referenceRound, evidenceRound \in Int:
      /\ referenceRound >= 0 /\ evidenceRound >= 0
      /\ LET reference == blockchain[CONFLICT_HEIGHT]
            referenceCommit == blockchain[CONFLICT_HEIGHT + 1].lastCommit
            evidenceHeader == conflictingBlock.header
            evidenceCommit == conflictingBlock.Commits
        IN
        IF ViolatesValidity(reference, evidenceHeader)
        THEN /\ attackers' := blockchain[COMMON_HEIGHT].NextVS \intersect evidenceCommit
             /\ state' := "Lunatic"
        ELSE IF referenceRound = evidenceRound
        THEN /\ attackers' := referenceCommit \intersect evidenceCommit
             /\ state' := "Equivocation"
        ELSE
          \* This property is shown in property
          \* Accountability of TendermintAcc3.tla
          /\ state' := "Amnesia"
          /\ \E Attackers \in SUBSET (Faulty \intersect reference.VS):
             /\ 3 * Cardinality(Attackers) > Cardinality(reference.VS)
             /\ attackers' := Attackers
    /\ blockchain' := blockchain
    /\ refClock' := refClock
    /\ Faulty' := Faulty
    /\ conflictingBlock' := conflictingBlock

(********************************** INVARIANTS *******************************)

\* This invariant ensure that the attackers have
\* more than 1/3 of the voting power
\*
\* [LCAI-INV-Output.1::TLA-DETECTION-COMPLETENESS.1]
DetectionCompleteness ==
  state /= "init" =>
    3 * Cardinality(attackers) > Cardinality(blockchain[CONFLICT_HEIGHT].VS)

\* This invariant ensures that only the faulty validators are detected
\*
\* [LCAI-INV-Output.1::TLA-DETECTION-ACCURACY.1]
DetectionAccuracy ==
  attackers \subseteq Faulty

==============================================================================
