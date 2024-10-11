-------------------------- MODULE Lightclient_002_draft ----------------------------
(**
 * A state-machine specification of the lite client, following the English spec:
 *
 * https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification.md
 *) 

EXTENDS Integers, FiniteSets

\* the parameters of Light Client
CONSTANTS
  TRUSTED_HEIGHT,
    (* an index of the block header that the light client trusts by social consensus *)
  TARGET_HEIGHT,
    (* an index of the block header that the light client tries to verify *)
  TRUSTING_PERIOD,
    (* the period within which the validators are trusted *)
  IS_PRIMARY_CORRECT
    (* is primary correct? *)  

VARIABLES       (* see TypeOK below for the variable types *)
  state,        (* the current state of the light client *)
  nextHeight,   (* the next height to explore by the light client *)
  nprobes       (* the lite client iteration, or the number of block tests *)
  
(* the light store *)
VARIABLES  
  fetchedLightBlocks, (* a function from heights to LightBlocks *)
  lightBlockStatus,   (* a function from heights to block statuses *)
  latestVerified      (* the latest verified block *)

(* the variables of the lite client *)
lcvars == <<state, nextHeight, fetchedLightBlocks, lightBlockStatus, latestVerified>>

(* the light client previous state components, used for monitoring *)
VARIABLES
  prevVerified,
  prevCurrent,
  prevNow,
  prevVerdict

InitMonitor(verified, current, now, verdict) ==
  /\ prevVerified = verified
  /\ prevCurrent = current
  /\ prevNow = now
  /\ prevVerdict = verdict

NextMonitor(verified, current, now, verdict) ==
  /\ prevVerified' = verified
  /\ prevCurrent' = current
  /\ prevNow' = now
  /\ prevVerdict' = verdict


(******************* Blockchain instance ***********************************)

\* the parameters that are propagated into Blockchain
CONSTANTS
  AllNodes
    (* a set of all nodes that can act as validators (correct and faulty) *)

\* the state variables of Blockchain, see Blockchain.tla for the details
VARIABLES now, blockchain, Faulty

\* All the variables of Blockchain. For some reason, BC!vars does not work
bcvars == <<now, blockchain, Faulty>>

(* Create an instance of Blockchain.
   We could write EXTENDS Blockchain, but then all the constants and state variables
   would be hidden inside the Blockchain module.
 *) 
ULTIMATE_HEIGHT == TARGET_HEIGHT + 1 
 
BC == INSTANCE Blockchain_002_draft WITH
  now <- now, blockchain <- blockchain, Faulty <- Faulty

(************************** Lite client ************************************)

(* the heights on which the light client is working *)  
HEIGHTS == TRUSTED_HEIGHT..TARGET_HEIGHT

(* the control states of the lite client *) 
States == { "working", "finishedSuccess", "finishedFailure" }

(**
 Check the precondition of ValidAndVerified.
 
 [LCV-FUNC-VALID.1::TLA-PRE.1]
 *)
ValidAndVerifiedPre(trusted, untrusted) ==
  LET thdr == trusted.header
      uhdr == untrusted.header
  IN
  /\ BC!InTrustingPeriod(thdr)
  /\ thdr.height < uhdr.height
     \* the trusted block has been created earlier (no drift here)
  /\ thdr.time < uhdr.time
     \* the untrusted block is not from the future
  /\ uhdr.time < now
  /\ untrusted.Commits \subseteq uhdr.VS
  /\ LET TP == Cardinality(uhdr.VS)
         SP == Cardinality(untrusted.Commits)
     IN
     3 * SP > 2 * TP     
  /\ thdr.height + 1 = uhdr.height => thdr.NextVS = uhdr.VS
  (* As we do not have explicit hashes we ignore these three checks of the English spec:
   
     1. "trusted.Commit is a commit is for the header trusted.Header,
      i.e. it contains the correct hash of the header".
     2. untrusted.Validators = hash(untrusted.Header.Validators)
     3. untrusted.NextValidators = hash(untrusted.Header.NextValidators)
   *)

(**
  * Check that the commits in an untrusted block form 1/3 of the next validators
  * in a trusted header.
 *)
SignedByOneThirdOfTrusted(trusted, untrusted) ==
  LET TP == Cardinality(trusted.header.NextVS)
      SP == Cardinality(untrusted.Commits \intersect trusted.header.NextVS)
  IN
  3 * SP > TP     

(**
 Check, whether an untrusted block is valid and verifiable w.r.t. a trusted header.
 
 [LCV-FUNC-VALID.1::TLA.1]
 *)   
ValidAndVerified(trusted, untrusted) ==
    IF ~ValidAndVerifiedPre(trusted, untrusted)
    THEN "INVALID"
    ELSE IF ~BC!InTrustingPeriod(untrusted.header)
    (* We leave the following test for the documentation purposes.
       The implementation should do this test, as signature verification may be slow.
       In the TLA+ specification, ValidAndVerified happens in no time.
     *) 
    THEN "FAILED_TRUSTING_PERIOD" 
    ELSE IF untrusted.header.height = trusted.header.height + 1
             \/ SignedByOneThirdOfTrusted(trusted, untrusted)
         THEN "SUCCESS"
         ELSE "NOT_ENOUGH_TRUST"

(*
 Initial states of the light client.
 Initially, only the trusted light block is present.
 *)
LCInit ==
    /\ state = "working"
    /\ nextHeight = TARGET_HEIGHT
    /\ nprobes = 0  \* no tests have been done so far
    /\ LET trustedBlock == blockchain[TRUSTED_HEIGHT]
           trustedLightBlock == [header |-> trustedBlock, Commits |-> AllNodes]
       IN
        \* initially, fetchedLightBlocks is a function of one element, i.e., TRUSTED_HEIGHT
        /\ fetchedLightBlocks = [h \in {TRUSTED_HEIGHT} |-> trustedLightBlock]
        \* initially, lightBlockStatus is a function of one element, i.e., TRUSTED_HEIGHT
        /\ lightBlockStatus = [h \in {TRUSTED_HEIGHT} |-> "StateVerified"]
        \* the latest verified block the the trusted block
        /\ latestVerified = trustedLightBlock
        /\ InitMonitor(trustedLightBlock, trustedLightBlock, now, "SUCCESS")

\* block should contain a copy of the block from the reference chain, with a matching commit
CopyLightBlockFromChain(block, height) ==
    LET ref == blockchain[height]
        lastCommit ==
          IF height < ULTIMATE_HEIGHT
          THEN blockchain[height + 1].lastCommit
            \* for the ultimate block, which we never use, as ULTIMATE_HEIGHT = TARGET_HEIGHT + 1
          ELSE blockchain[height].VS 
    IN
    block = [header |-> ref, Commits |-> lastCommit]      

\* Either the primary is correct and the block comes from the reference chain,
\* or the block is produced by a faulty primary.
\*
\* [LCV-FUNC-FETCH.1::TLA.1]
FetchLightBlockInto(block, height) ==
    IF IS_PRIMARY_CORRECT
    THEN CopyLightBlockFromChain(block, height)
    ELSE BC!IsLightBlockAllowedByDigitalSignatures(height, block)

\* add a block into the light store    
\*
\* [LCV-FUNC-UPDATE.1::TLA.1]
LightStoreUpdateBlocks(lightBlocks, block) ==
    LET ht == block.header.height IN    
    [h \in DOMAIN lightBlocks \union {ht} |->
        IF h = ht THEN block ELSE lightBlocks[h]]

\* update the state of a light block      
\*
\* [LCV-FUNC-UPDATE.1::TLA.1]
LightStoreUpdateStates(statuses, ht, blockState) ==
    [h \in DOMAIN statuses \union {ht} |->
        IF h = ht THEN blockState ELSE statuses[h]]      

\* Check, whether newHeight is a possible next height for the light client.
\*
\* [LCV-FUNC-SCHEDULE.1::TLA.1]
CanScheduleTo(newHeight, pLatestVerified, pNextHeight, pTargetHeight) ==
    LET ht == pLatestVerified.header.height IN
    \/ /\ ht = pNextHeight
       /\ ht < pTargetHeight
       /\ pNextHeight < newHeight
       /\ newHeight <= pTargetHeight
    \/ /\ ht < pNextHeight
       /\ ht < pTargetHeight
       /\ ht < newHeight
       /\ newHeight < pNextHeight
    \/ /\ ht = pTargetHeight
       /\ newHeight = pTargetHeight

\* The loop of VerifyToTarget.
\*
\* [LCV-FUNC-MAIN.1::TLA-LOOP.1]
VerifyToTargetLoop ==
      \* the loop condition is true
    /\ latestVerified.header.height < TARGET_HEIGHT
      \* pick a light block, which will be constrained later
    /\ \E current \in BC!LightBlocks:
        \* Get next LightBlock for verification
        /\ IF nextHeight \in DOMAIN fetchedLightBlocks
           THEN \* copy the block from the light store
                /\ current = fetchedLightBlocks[nextHeight]
                /\ UNCHANGED fetchedLightBlocks
           ELSE \* retrieve a light block and save it in the light store
                /\ FetchLightBlockInto(current, nextHeight)
                /\ fetchedLightBlocks' = LightStoreUpdateBlocks(fetchedLightBlocks, current)
        \* Record that one more probe has been done (for complexity and model checking)
        /\ nprobes' = nprobes + 1
        \* Verify the current block
        /\ LET verdict == ValidAndVerified(latestVerified, current) IN
           NextMonitor(latestVerified, current, now, verdict) /\
           \* Decide whether/how to continue
           CASE verdict = "SUCCESS" ->
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateVerified")
              /\ latestVerified' = current
              /\ state' =
                    IF latestVerified'.header.height < TARGET_HEIGHT
                    THEN "working"
                    ELSE "finishedSuccess"
              /\ \E newHeight \in HEIGHTS:
                 /\ CanScheduleTo(newHeight, current, nextHeight, TARGET_HEIGHT)
                 /\ nextHeight' = newHeight
                  
           [] verdict = "NOT_ENOUGH_TRUST" ->
              (*
                do nothing: the light block current passed validation, but the validator
                set is too different to verify it. We keep the state of
                current at StateUnverified. For a later iteration, Schedule
                might decide to try verification of that light block again.
                *)
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateUnverified")
              /\ \E newHeight \in HEIGHTS:
                 /\ CanScheduleTo(newHeight, latestVerified, nextHeight, TARGET_HEIGHT)
                 /\ nextHeight' = newHeight 
              /\ UNCHANGED <<latestVerified, state>>
              
           [] OTHER ->
              \* verdict is some error code
              /\ lightBlockStatus' = LightStoreUpdateStates(lightBlockStatus, nextHeight, "StateFailed")
              /\ state' = "finishedFailure"
              /\ UNCHANGED <<latestVerified, nextHeight>>

\* The terminating condition of VerifyToTarget.
\*
\* [LCV-FUNC-MAIN.1::TLA-LOOPCOND.1]
VerifyToTargetDone ==
    /\ latestVerified.header.height >= TARGET_HEIGHT
    /\ state' = "finishedSuccess"
    /\ UNCHANGED <<nextHeight, nprobes, fetchedLightBlocks, lightBlockStatus, latestVerified>>
    /\ UNCHANGED <<prevVerified, prevCurrent, prevNow, prevVerdict>>
            
(********************* Lite client + Blockchain *******************)
Init ==
    \* the blockchain is initialized immediately to the ULTIMATE_HEIGHT
    /\ BC!InitToHeight
    \* the light client starts
    /\ LCInit

(*
  The system step is very simple.
  The light client is either executing VerifyToTarget, or it has terminated.
  (In the latter case, a model checker reports a deadlock.)
  Simultaneously, the global clock may advance.
 *)
Next ==
    /\ state = "working"
    /\ VerifyToTargetLoop \/ VerifyToTargetDone 
    /\ BC!AdvanceTime \* the global clock is advanced by zero or more time units

(************************* Types ******************************************)
TypeOK ==
    /\ state \in States
    /\ nextHeight \in HEIGHTS
    /\ latestVerified \in BC!LightBlocks
    /\ \E HS \in SUBSET HEIGHTS:
        /\ fetchedLightBlocks \in [HS -> BC!LightBlocks]
        /\ lightBlockStatus
             \in [HS -> {"StateVerified", "StateUnverified", "StateFailed"}]

(************************* Properties ******************************************)

(* The properties to check *)
\* this invariant candidate is false    
NeverFinish ==
    state = "working"

\* this invariant candidate is false    
NeverFinishNegative ==
    state /= "finishedFailure"

\* This invariant holds true, when the primary is correct. 
\* This invariant candidate is false when the primary is faulty.    
NeverFinishNegativeWhenTrusted ==
    (*(minTrustedHeight <= TRUSTED_HEIGHT)*)
    BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
      => state /= "finishedFailure"     

\* this invariant candidate is false    
NeverFinishPositive ==
    state /= "finishedSuccess"

(**
  Correctness states that all the obtained headers are exactly like in the blockchain.
 
  It is always the case that every verified header in LightStore was generated by
  an instance of Tendermint consensus.
  
  [LCV-DIST-SAFE.1::CORRECTNESS-INV.1]
 *)  
CorrectnessInv ==
    \A h \in DOMAIN fetchedLightBlocks:
        lightBlockStatus[h] = "StateVerified" =>
            fetchedLightBlocks[h].header = blockchain[h]

(**
 Check that the sequence of the headers in storedLightBlocks satisfies ValidAndVerified = "SUCCESS" pairwise
 This property is easily violated, whenever a header cannot be trusted anymore.
 *)
StoredHeadersAreVerifiedInv ==
    state = "finishedSuccess"
        =>
        \A lh, rh \in DOMAIN fetchedLightBlocks: \* for every pair of different stored headers
            \/ lh >= rh
               \* either there is a header between them
            \/ \E mh \in DOMAIN fetchedLightBlocks:
                lh < mh /\ mh < rh
               \* or we can verify the right one using the left one
            \/ "SUCCESS" = ValidAndVerified(fetchedLightBlocks[lh], fetchedLightBlocks[rh])

\* An improved version of StoredHeadersAreSound, assuming that a header may be not trusted.
\* This invariant candidate is also violated,
\* as there may be some unverified blocks left in the middle.
StoredHeadersAreVerifiedOrNotTrustedInv ==
    state = "finishedSuccess"
        =>
        \A lh, rh \in DOMAIN fetchedLightBlocks: \* for every pair of different stored headers
            \/ lh >= rh
               \* either there is a header between them
            \/ \E mh \in DOMAIN fetchedLightBlocks:
                lh < mh /\ mh < rh
               \* or we can verify the right one using the left one
            \/ "SUCCESS" = ValidAndVerified(fetchedLightBlocks[lh], fetchedLightBlocks[rh])
               \* or the left header is outside the trusting period, so no guarantees
            \/ ~BC!InTrustingPeriod(fetchedLightBlocks[lh].header) 

(**
 * An improved version of StoredHeadersAreSoundOrNotTrusted,
 * checking the property only for the verified headers.
 * This invariant holds true.
 *)
ProofOfChainOfTrustInv ==
    state = "finishedSuccess"
        =>
        \A lh, rh \in DOMAIN fetchedLightBlocks:
                \* for every pair of stored headers that have been verified
            \/ lh >= rh
            \/ lightBlockStatus[lh] = "StateUnverified"
            \/ lightBlockStatus[rh] = "StateUnverified"
               \* either there is a header between them
            \/ \E mh \in DOMAIN fetchedLightBlocks:
                lh < mh /\ mh < rh /\ lightBlockStatus[mh] = "StateVerified"
               \* or the left header is outside the trusting period, so no guarantees
            \/ ~(BC!InTrustingPeriod(fetchedLightBlocks[lh].header))
               \* or we can verify the right one using the left one
            \/ "SUCCESS" = ValidAndVerified(fetchedLightBlocks[lh], fetchedLightBlocks[rh])

(**
 * When the light client terminates, there are no failed blocks. (Otherwise, someone lied to us.) 
 *)            
NoFailedBlocksOnSuccessInv ==
    state = "finishedSuccess" =>
        \A h \in DOMAIN fetchedLightBlocks:
            lightBlockStatus[h] /= "StateFailed"            

\* This property states that whenever the light client finishes with a positive outcome,
\* the trusted header is still within the trusting period.
\* We expect this property to be violated. And Apalache shows us a counterexample.
PositiveBeforeTrustedHeaderExpires ==
    (state = "finishedSuccess") => BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
    
\* If the primary is correct and the initial trusted block has not expired,
\* then whenever the algorithm terminates, it reports "success"    
CorrectPrimaryAndTimeliness ==
  (BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
    /\ state /= "working" /\ IS_PRIMARY_CORRECT) =>
      state = "finishedSuccess"     

(**
  If the primary is correct and there is a trusted block that has not expired,
  then whenever the algorithm terminates, it reports "success".

  [LCV-DIST-LIVE.1::SUCCESS-CORR-PRIMARY-CHAIN-OF-TRUST.1]
 *)
SuccessOnCorrectPrimaryAndChainOfTrust ==
  (\E h \in DOMAIN fetchedLightBlocks:
        lightBlockStatus[h] = "StateVerified" /\ BC!InTrustingPeriod(blockchain[h])
    /\ state /= "working" /\ IS_PRIMARY_CORRECT) =>
      state = "finishedSuccess"     

\* Lite Client Completeness: If header h was correctly generated by an instance
\* of Tendermint consensus (and its age is less than the trusting period),
\* then the lite client should eventually set trust(h) to true.
\*
\* Note that Completeness assumes that the lite client communicates with a correct full node.
\*
\* We decompose completeness into Termination (liveness) and Precision (safety).
\* Once again, Precision is an inverse version of the safety property in Completeness,
\* as A => B is logically equivalent to ~B => ~A. 
PrecisionInv ==
    (state = "finishedFailure")
      => \/ ~BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT]) \* outside of the trusting period
         \/ \E h \in DOMAIN fetchedLightBlocks:
            LET lightBlock == fetchedLightBlocks[h] IN
                 \* the full node lied to the lite client about the block header
              \/ lightBlock.header /= blockchain[h]
                 \* the full node lied to the lite client about the commits
              \/ lightBlock.Commits /= lightBlock.header.VS

\* the old invariant that was found to be buggy by TLC
PrecisionBuggyInv ==
    (state = "finishedFailure")
      => \/ ~BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT]) \* outside of the trusting period
         \/ \E h \in DOMAIN fetchedLightBlocks:
            LET lightBlock == fetchedLightBlocks[h] IN
            \* the full node lied to the lite client about the block header
            lightBlock.header /= blockchain[h]

\* the worst complexity
Complexity ==
    LET N == TARGET_HEIGHT - TRUSTED_HEIGHT + 1 IN
    state /= "working" =>
        (2 * nprobes <= N * (N - 1)) 

(*
 We omit termination, as the algorithm deadlocks in the end.
 So termination can be demonstrated by finding a deadlock.
 Of course, one has to analyze the deadlocked state and see that
 the algorithm has indeed terminated there.
*)
=============================================================================
\* Modification History
\* Last modified Fri Jun 26 12:08:28 CEST 2020 by igor
\* Created Wed Oct 02 16:39:42 CEST 2019 by igor
