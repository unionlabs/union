-------------------------- MODULE Lightclient_003_draft ----------------------------
(**
 * A state-machine specification of the lite client verification,
 * following the English spec:
 *
 * https://github.com/informalsystems/tendermint-rs/blob/master/docs/spec/lightclient/verification.md
 *) 

EXTENDS Integers, FiniteSets, typedefs

\* the parameters of Light Client
CONSTANTS
  \* an index of the block header that the light client trusts by social consensus
  \* @type: Int;
  TRUSTED_HEIGHT,
  \* an index of the block header that the light client tries to verify
  \* @type: Int;
  TARGET_HEIGHT,
  \* the period within which the validators are trusted
  \* @type: Int;
  TRUSTING_PERIOD,
  \* the assumed precision of the clock
  \* @type: Int;
  CLOCK_DRIFT,
  \* the actual clock drift, which under normal circumstances should not
  \* be larger than CLOCK_DRIFT (otherwise, there will be a bug)
  \*
  \* @type: Int;
  REAL_CLOCK_DRIFT,
  \* is primary correct?
  \* @type: Bool;
  IS_PRIMARY_CORRECT,
  \* a pair <<a, b>> that limits that ratio of faulty validator in the blockchain
  \* from above (exclusive). Cosmos security model prescribes 1 / 3
  \* @type: <<Int, Int>>;
  FAULTY_RATIO

VARIABLES
  \* current time as measured by the light client
  \* @type: Int;
  localClock,
  \* the current state of the light client
  \* @type: Str;
  state,
  \* the next height to explore by the light client
  \* @type: Int;
  nextHeight,
  \* the lite client iteration, or the number of block tests
  \* @type: Int;
  nprobes
  
(* the light store *)
VARIABLES  
  \* a function from heights to LightBlocks
  \* @type: Int -> $lightHeader;
  fetchedLightBlocks,
  \* a function from heights to block statuses
  \* @type: Int -> Str;
  lightBlockStatus,
  \* the latest verified block
  \* @type: $lightHeader;
  latestVerified

(* the variables of the lite client *)
lcvars == <<localClock, state, nextHeight,
            fetchedLightBlocks, lightBlockStatus, latestVerified>>

(* the light client previous state components, used for monitoring *)
VARIABLES
  \* @type: $lightHeader;
  prevVerified,
  \* @type: $lightHeader;
  prevCurrent,
  \* @type: Int;
  prevLocalClock,
  \* @type: Str;
  prevVerdict

InitMonitor(verified, current, pLocalClock, verdict) ==
  /\ prevVerified = verified
  /\ prevCurrent = current
  /\ prevLocalClock = pLocalClock
  /\ prevVerdict = verdict

NextMonitor(verified, current, pLocalClock, verdict) ==
  /\ prevVerified' = verified
  /\ prevCurrent' = current
  /\ prevLocalClock' = pLocalClock
  /\ prevVerdict' = verdict


(******************* Blockchain instance ***********************************)

\* the parameters that are propagated into Blockchain
CONSTANTS
  \* a set of all nodes that can act as validators (correct and faulty)
  \* @type: Set($node);
  AllNodes

\* the state variables of Blockchain, see Blockchain.tla for the details
VARIABLES
    \* the current global time in integer units as perceived by the reference chain
    \* @type: Int;
    refClock,
    \* A sequence of BlockHeaders, which gives us a bird view of the blockchain
    \* @type: Int -> $header;
    blockchain,
    \* A set of faulty nodes, which can act as validators.
    \* We assume that the set of faulty processes is non-decreasing.
    \* If a process has recovered, it should connect using a different id.
    \* @type: Set($node);
    Faulty

\* All the variables of Blockchain. For some reason, BC!vars does not work
bcvars == <<refClock, blockchain, Faulty>>

(* Create an instance of Blockchain.
   We could write EXTENDS Blockchain, but then all the constants and state variables
   would be hidden inside the Blockchain module.
 *) 
ULTIMATE_HEIGHT == TARGET_HEIGHT + 1 
 
BC == INSTANCE Blockchain_003_draft WITH
  refClock <- refClock, blockchain <- blockchain, Faulty <- Faulty

(************************** Lite client ************************************)

(* the heights on which the light client is working *)  
HEIGHTS == TRUSTED_HEIGHT..TARGET_HEIGHT

(* the control states of the lite client *) 
States == { "working", "finishedSuccess", "finishedFailure" }

\* The verification functions are implemented in the API
API == INSTANCE LCVerificationApi_003_draft


(*
 Initial states of the light client.
 Initially, only the trusted light block is present.
 *)
LCInit ==
    /\ \E tm \in Int:
        tm >= 0 /\ API!IsLocalClockWithinDrift(tm, refClock) /\ localClock = tm
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
        /\ InitMonitor(trustedLightBlock, trustedLightBlock, localClock, "SUCCESS")

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
\*
\* @type: (Int -> $lightHeader, $lightHeader) => (Int -> $lightHeader);
LightStoreUpdateBlocks(lightBlocks, block) ==
    LET ht == block.header.height IN    
    [h \in DOMAIN lightBlocks \union {ht} |->
        IF h = ht THEN block ELSE lightBlocks[h]]

\* update the state of a light block      
\*
\* [LCV-FUNC-UPDATE.1::TLA.1]
\*
\* @type: (Int -> Str, Int, Str) => (Int -> Str);
LightStoreUpdateStates(statuses, ht, blockState) ==
    [h \in DOMAIN statuses \union {ht} |->
        IF h = ht THEN blockState ELSE statuses[h]]      

\* Check, whether newHeight is a possible next height for the light client.
\*
\* [LCV-FUNC-SCHEDULE.1::TLA.1]
\*
\* @type: (Int, $lightHeader, Int, Int) => Bool;
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
        /\ LET verdict == API!ValidAndVerified(latestVerified, current, TRUE) IN
           NextMonitor(latestVerified, current, localClock, verdict) /\
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
    /\ UNCHANGED <<prevVerified, prevCurrent, prevLocalClock, prevVerdict>>

(*
  The local and global clocks can be updated. They can also drift from each other.
  Note that the local clock can actually go backwards in time.
  However, it still stays in the drift envelope
  of [refClock - REAL_CLOCK_DRIFT, refClock + REAL_CLOCK_DRIFT].
 *)
AdvanceClocks ==
    /\ BC!AdvanceTime
    /\ \E tm \in Int:
        /\ tm >= 0
        /\ API!IsLocalClockWithinDrift(tm, refClock')
        /\ localClock' = tm
        \* if you like the clock to always grow monotonically, uncomment the next line:
        \*/\ localClock' > localClock
            
(********************* Lite client + Blockchain *******************)
Init ==
    \* the blockchain is initialized immediately to the ULTIMATE_HEIGHT
    /\ BC!InitToHeight(FAULTY_RATIO)
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
    /\ AdvanceClocks

(************************* Types ******************************************)
TypeOK ==
    /\ state \in States
    /\ localClock \in Nat
    /\ refClock \in Nat
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
    BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
      => state /= "finishedFailure"     

\* this invariant candidate is false    
NeverFinishPositive ==
    state /= "finishedSuccess"


(**
 Check that the target height has been reached upon successful termination.
 *)
TargetHeightOnSuccessInv ==
    state = "finishedSuccess" =>
        /\ TARGET_HEIGHT \in DOMAIN fetchedLightBlocks
        /\ lightBlockStatus[TARGET_HEIGHT] = "StateVerified"

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
 No faulty block was used to construct a proof. This invariant holds,
 only if FAULTY_RATIO < 1/3.
 *)
NoTrustOnFaultyBlockInv ==
    (state = "finishedSuccess"
        /\ fetchedLightBlocks[TARGET_HEIGHT].header = blockchain[TARGET_HEIGHT])
        => CorrectnessInv

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
            \/ "SUCCESS" = API!ValidAndVerified(fetchedLightBlocks[lh],
                                                fetchedLightBlocks[rh], FALSE)

\* An improved version of StoredHeadersAreVerifiedInv,
\* assuming that a header may be not trusted.
\* This invariant candidate is also violated,
\* as there may be some unverified blocks left in the middle.
\* This property is violated under two conditions:
\* (1) the primary is faulty and there are at least 4 blocks,
\* (2) the primary is correct and there are at least 5 blocks.
StoredHeadersAreVerifiedOrNotTrustedInv ==
    state = "finishedSuccess"
        =>
        \A lh, rh \in DOMAIN fetchedLightBlocks: \* for every pair of different stored headers
            \/ lh >= rh
               \* either there is a header between them
            \/ \E mh \in DOMAIN fetchedLightBlocks:
                lh < mh /\ mh < rh
               \* or we can verify the right one using the left one
            \/ "SUCCESS" = API!ValidAndVerified(fetchedLightBlocks[lh],
                                                fetchedLightBlocks[rh], FALSE)
               \* or the left header is outside the trusting period, so no guarantees
            \/ ~API!InTrustingPeriodLocal(fetchedLightBlocks[lh].header) 

(**
 * An improved version of StoredHeadersAreSoundOrNotTrusted,
 * checking the property only for the verified headers.
 * This invariant holds true if CLOCK_DRIFT <= REAL_CLOCK_DRIFT.
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
            \/ ~(API!InTrustingPeriodLocal(fetchedLightBlocks[lh].header))
               \* or we can verify the right one using the left one
            \/ "SUCCESS" = API!ValidAndVerified(fetchedLightBlocks[lh],
                                                fetchedLightBlocks[rh], FALSE)

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
    (state = "finishedSuccess") =>
        BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
    
\* If the primary is correct and the initial trusted block has not expired,
\* then whenever the algorithm terminates, it reports "success".
\* This property fails.
CorrectPrimaryAndTimeliness ==
  (BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
    /\ state /= "working" /\ IS_PRIMARY_CORRECT) =>
      state = "finishedSuccess"     

(**
  If the primary is correct and there is a trusted block that has not expired,
  then whenever the algorithm terminates, it reports "success".
  This property only holds true, if the local clock is always growing monotonically.
  If the local clock can go backwards in the envelope
  [refClock - CLOCK_DRIFT, refClock + CLOCK_DRIFT], then the property fails.

  [LCV-DIST-LIVE.1::SUCCESS-CORR-PRIMARY-CHAIN-OF-TRUST.1]
 *)
SuccessOnCorrectPrimaryAndChainOfTrustLocal ==
  (\E h \in DOMAIN fetchedLightBlocks:
        /\ lightBlockStatus[h] = "StateVerified"
        /\ API!InTrustingPeriodLocal(blockchain[h])
    /\ state /= "working" /\ IS_PRIMARY_CORRECT) =>
      state = "finishedSuccess"     

(**
  Similar to SuccessOnCorrectPrimaryAndChainOfTrust, but using the blockchain clock.
  It fails because the local clock of the client drifted away, so it rejects a block
  that has not expired yet (according to the local clock).
 *)
SuccessOnCorrectPrimaryAndChainOfTrustGlobal ==
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
\*
\* This property holds only when CLOCK_DRIFT = 0 and REAL_CLOCK_DRIFT = 0.
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

(**
 If the light client has terminated, then the expected postcondition holds true.
 *)
ApiPostInv ==
    state /= "working" =>
        API!VerifyToTargetPost(blockchain, IS_PRIMARY_CORRECT,
                fetchedLightBlocks, lightBlockStatus,
                TRUSTED_HEIGHT, TARGET_HEIGHT, state)

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
