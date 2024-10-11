-------------------------- MODULE LCDetector_003_draft -----------------------------
(**
 * This is a specification of the light client detector module.
 * It follows the English specification:
 *
 * https://github.com/cometbft/cometbft/blob/main/spec/light-client/detection/detection_003_reviewed.md
 *
 * The assumptions made in this specification:
 *
 *  - light client connects to one primary and one secondary peer
 *
 *  - the light client has its own local clock that can drift from the reference clock
 *    within the envelope [refClock - CLOCK_DRIFT, refClock + CLOCK_DRIFT].
 *    The local clock may increase as well as decrease in the the envelope
 *    (similar to clock synchronization).
 *
 *  - the ratio of the faulty validators is set as the parameter.
 *
 * Igor Konnov, Josef Widder, 2020
 *)

EXTENDS Integers

\* the parameters of Light Client
CONSTANTS
  AllNodes,
    (* a set of all nodes that can act as validators (correct and faulty) *)
  TRUSTED_HEIGHT,
    (* an index of the block header that the light client trusts by social consensus *)
  TARGET_HEIGHT,
    (* an index of the block header that the light client tries to verify *)
  TRUSTING_PERIOD,
    (* the period within which the validators are trusted *)
  CLOCK_DRIFT,
    (* the assumed precision of the clock *)
  REAL_CLOCK_DRIFT,
    (* the actual clock drift, which under normal circumstances should not
       be larger than CLOCK_DRIFT (otherwise, there will be a bug) *)
  FAULTY_RATIO,
    (* a pair <<a, b>> that limits that ratio of faulty validator in the blockchain
       from above (exclusive). Cosmos security model prescribes 1 / 3. *)
  IS_PRIMARY_CORRECT,
  IS_SECONDARY_CORRECT

VARIABLES
  blockchain,           (* the reference blockchain *)
  localClock,           (* the local clock of the light client *)
  refClock,             (* the reference clock in the reference blockchain *)
  Faulty,               (* the set of faulty validators *)
  state,                (* the state of the light client detector *)
  fetchedLightBlocks1,  (* a function from heights to LightBlocks *)
  fetchedLightBlocks2,  (* a function from heights to LightBlocks *)
  fetchedLightBlocks1b, (* a function from heights to LightBlocks *)
  commonHeight,         (* the height that is trusted in CreateEvidenceForPeer *)
  nextHeightToTry,      (* the index in CreateEvidenceForPeer *)
  evidences             (* a set of evidences *)

vars == <<state, blockchain, localClock, refClock, Faulty,
          fetchedLightBlocks1, fetchedLightBlocks2, fetchedLightBlocks1b,
          commonHeight, nextHeightToTry, evidences >>

\* (old) type annotations in Apalache
a <: b == a

 
\* instantiate a reference chain
ULTIMATE_HEIGHT == TARGET_HEIGHT + 1 
BC == INSTANCE Blockchain_003_draft
    WITH ULTIMATE_HEIGHT <- (TARGET_HEIGHT + 1)

\* use the light client API
LC == INSTANCE LCVerificationApi_003_draft

\* evidence type
ET == [peer |-> STRING, conflictingBlock |-> BC!LBT, commonHeight |-> Int]

\* is the algorithm in the terminating state
IsTerminated ==
    state \in { <<"NoEvidence", "PRIMARY">>,
                <<"NoEvidence", "SECONDARY">>,
                <<"FaultyPeer", "PRIMARY">>,
                <<"FaultyPeer", "SECONDARY">>,
                <<"FoundEvidence", "PRIMARY">> }


(********************************* Initialization ******************************)

\* initialization for the light blocks data structure
InitLightBlocks(lb, Heights) ==
    \* BC!LightBlocks is an infinite set, as time is not restricted.
    \* Hence, we initialize the light blocks by picking the sets inside.
    \E vs, nextVS, lastCommit, commit \in [Heights -> SUBSET AllNodes]:
      \* although [Heights -> Int] is an infinite set,
      \* Apalache needs just one instance of this set, so it does not complain.
      \E timestamp \in [Heights -> Int]:
        LET hdr(h) ==
             [height |-> h,
              time |-> timestamp[h],
              VS |-> vs[h],
              NextVS |-> nextVS[h],
              lastCommit |-> lastCommit[h]]
        IN
        LET lightHdr(h) ==
            [header |-> hdr(h), Commits |-> commit[h]]
        IN
        lb = [ h \in Heights |-> lightHdr(h) ]

\* initialize the detector algorithm
Init ==
    \* initialize the blockchain to TARGET_HEIGHT + 1
    /\ BC!InitToHeight(FAULTY_RATIO)
    /\ \E tm \in Int:
        tm >= 0 /\ LC!IsLocalClockWithinDrift(tm, refClock) /\ localClock = tm
    \* start with the secondary looking for evidence
    /\ state = <<"Init", "SECONDARY">> /\ commonHeight = 0 /\ nextHeightToTry = 0
    /\ evidences = {} <: {ET}
    \* Precompute a possible result of light client verification for the primary.
    \* It is the input to the detection algorithm.
    /\ \E Heights1 \in SUBSET(TRUSTED_HEIGHT..TARGET_HEIGHT):
        /\ TRUSTED_HEIGHT \in Heights1
        /\ TARGET_HEIGHT \in Heights1
        /\ InitLightBlocks(fetchedLightBlocks1, Heights1)
        \* As we have a non-deterministic scheduler, for every trace that has
        \* an unverified block, there is a filtered trace that only has verified
        \* blocks. This is a deep observation.
        /\ LET status == [h \in Heights1 |-> "StateVerified"] IN
           LC!VerifyToTargetPost(blockchain, IS_PRIMARY_CORRECT,
                                 fetchedLightBlocks1, status,
                                 TRUSTED_HEIGHT, TARGET_HEIGHT, "finishedSuccess")
    \* initialize the other data structures to the default values
    /\ LET trustedBlock == blockchain[TRUSTED_HEIGHT]
           trustedLightBlock == [header |-> trustedBlock, Commits |-> AllNodes]
       IN
       /\ fetchedLightBlocks2 =  [h \in {TRUSTED_HEIGHT} |-> trustedLightBlock]
       /\ fetchedLightBlocks1b = [h \in {TRUSTED_HEIGHT} |-> trustedLightBlock]


(********************************* Transitions ******************************)

\* a block should contain a copy of the block from the reference chain,
\* with a matching commit
CopyLightBlockFromChain(block, height) ==
    LET ref == blockchain[height]
        lastCommit ==
          IF height < ULTIMATE_HEIGHT
          THEN blockchain[height + 1].lastCommit
            \* for the ultimate block, which we never use,
            \* as ULTIMATE_HEIGHT = TARGET_HEIGHT + 1
          ELSE blockchain[height].VS 
    IN
    block = [header |-> ref, Commits |-> lastCommit]      

\* Either the primary is correct and the block comes from the reference chain,
\* or the block is produced by a faulty primary.
\*
\* [LCV-FUNC-FETCH.1::TLA.1]
FetchLightBlockInto(isPeerCorrect, block, height) ==
    IF isPeerCorrect
    THEN CopyLightBlockFromChain(block, height)
    ELSE BC!IsLightBlockAllowedByDigitalSignatures(height, block)


(**
 * Pick the next height, for which there is a block.
 *)
PickNextHeight(fetchedBlocks, height) ==
    LET largerHeights == { h \in DOMAIN fetchedBlocks: h > height } IN
    IF largerHeights = ({} <: {Int})
    THEN -1
    ELSE CHOOSE h \in largerHeights:
            \A h2 \in largerHeights: h <= h2


(**
 * Check, whether the target header matches at the secondary and primary.
 *)
CompareLast ==
    /\ state = <<"Init", "SECONDARY">>
    \* fetch a block from the secondary:
    \* non-deterministically pick a block that matches the constraints
    /\ \E latest \in BC!LightBlocks:
        \* for the moment, we ignore the possibility of a timeout when fetching a block
        /\ FetchLightBlockInto(IS_SECONDARY_CORRECT, latest, TARGET_HEIGHT)
        /\  IF latest.header = fetchedLightBlocks1[TARGET_HEIGHT].header
            THEN \* if the headers match, CreateEvidence is not called
                 /\ state' = <<"NoEvidence", "SECONDARY">>
                 \* save the retrieved block for further analysis
                 /\ fetchedLightBlocks2' =
                        [h \in (DOMAIN fetchedLightBlocks2) \union {TARGET_HEIGHT} |->
                            IF h = TARGET_HEIGHT THEN latest ELSE fetchedLightBlocks2[h]]
                 /\ UNCHANGED <<commonHeight, nextHeightToTry>>
            ELSE \* prepare the parameters for CreateEvidence
                 /\ commonHeight' = TRUSTED_HEIGHT
                 /\ nextHeightToTry' = PickNextHeight(fetchedLightBlocks1, TRUSTED_HEIGHT)
                 /\ state' = IF nextHeightToTry' >= 0
                             THEN <<"CreateEvidence", "SECONDARY">>
                             ELSE <<"FaultyPeer", "SECONDARY">>
                 /\ UNCHANGED fetchedLightBlocks2

    /\ UNCHANGED <<blockchain, Faulty,
                   fetchedLightBlocks1, fetchedLightBlocks1b, evidences>>


\* the actual loop in CreateEvidence
CreateEvidence(peer, isPeerCorrect, refBlocks, targetBlocks) ==
    /\ state = <<"CreateEvidence", peer>>
    \* precompute a possible result of light client verification for the secondary
    \* we have to introduce HeightRange, because Apalache can only handle a..b
    \* for constant a and b
    /\ LET HeightRange == { h \in TRUSTED_HEIGHT..TARGET_HEIGHT:
                                commonHeight <= h /\ h <= nextHeightToTry } IN
      \E HeightsRange \in SUBSET(HeightRange):
        /\ commonHeight \in HeightsRange /\ nextHeightToTry \in HeightsRange
        /\ InitLightBlocks(targetBlocks, HeightsRange)
        \* As we have a non-deterministic scheduler, for every trace that has
        \* an unverified block, there is a filtered trace that only has verified
        \* blocks. This is a deep observation.
        /\ \E result \in {"finishedSuccess", "finishedFailure"}:
            LET targetStatus == [h \in HeightsRange |-> "StateVerified"] IN
            \* call VerifyToTarget for (commonHeight, nextHeightToTry).
            /\ LC!VerifyToTargetPost(blockchain, isPeerCorrect,
                                     targetBlocks, targetStatus,
                                     commonHeight, nextHeightToTry, result)
               \* case 1: the peer has failed (or the trusting period has expired)
            /\ \/ /\ result /= "finishedSuccess"
                  /\ state' = <<"FaultyPeer", peer>>
                  /\ UNCHANGED <<commonHeight, nextHeightToTry, evidences>>
               \* case 2: success
               \/ /\ result = "finishedSuccess"
                  /\ LET block1 == refBlocks[nextHeightToTry] IN
                     LET block2 == targetBlocks[nextHeightToTry] IN
                     IF block1.header /= block2.header
                     THEN \* the target blocks do not match
                       /\ state' = <<"FoundEvidence", peer>>
                       /\ evidences' = evidences \union
                            {[peer |-> peer,
                              conflictingBlock |-> block1,
                              commonHeight |-> commonHeight]}
                       /\ UNCHANGED <<commonHeight, nextHeightToTry>>
                     ELSE \* the target blocks match
                       /\ nextHeightToTry' = PickNextHeight(refBlocks, nextHeightToTry)
                       /\ commonHeight' = nextHeightToTry
                       /\ state' = IF nextHeightToTry' >= 0
                                   THEN state
                                   ELSE <<"NoEvidence", peer>>
                       /\ UNCHANGED evidences  

SwitchToPrimary ==
    /\ state = <<"FoundEvidence", "SECONDARY">>
    /\ nextHeightToTry' = PickNextHeight(fetchedLightBlocks2, commonHeight)
    /\ state' = <<"CreateEvidence", "PRIMARY">>
    /\ UNCHANGED <<blockchain, refClock, Faulty, localClock,
                   fetchedLightBlocks1, fetchedLightBlocks2, fetchedLightBlocks1b,
                   commonHeight, evidences >>


CreateEvidenceForSecondary ==
  /\ CreateEvidence("SECONDARY", IS_SECONDARY_CORRECT,
                    fetchedLightBlocks1, fetchedLightBlocks2')
  /\ UNCHANGED <<blockchain, refClock, Faulty, localClock,
        fetchedLightBlocks1, fetchedLightBlocks1b>>

CreateEvidenceForPrimary ==
  /\ CreateEvidence("PRIMARY", IS_PRIMARY_CORRECT,
                    fetchedLightBlocks2,
                    fetchedLightBlocks1b')
  /\ UNCHANGED <<blockchain, Faulty,
        fetchedLightBlocks1, fetchedLightBlocks2>>

(*
  The local and global clocks can be updated. They can also drift from each other.
  Note that the local clock can actually go backwards in time.
  However, it still stays in the drift envelope
  of [refClock - REAL_CLOCK_DRIFT, refClock + REAL_CLOCK_DRIFT].
 *)
AdvanceClocks ==
    /\ \E tm \in Int:
        tm >= refClock /\ refClock' = tm
    /\ \E tm \in Int:
        /\ tm >= localClock
        /\ LC!IsLocalClockWithinDrift(tm, refClock')
        /\ localClock' = tm
   
(**
 Execute AttackDetector for one secondary.

 [LCD-FUNC-DETECTOR.2::LOOP.1]
 *)
Next ==
    /\ AdvanceClocks
    /\ \/ CompareLast
       \/ CreateEvidenceForSecondary
       \/ SwitchToPrimary 
       \/ CreateEvidenceForPrimary


\* simple invariants to see the progress of the detector
NeverNoEvidence == state[1] /= "NoEvidence"
NeverFoundEvidence == state[1] /= "FoundEvidence"
NeverFaultyPeer == state[1] /= "FaultyPeer"
NeverCreateEvidence == state[1] /= "CreateEvidence"

NeverFoundEvidencePrimary == state /= <<"FoundEvidence", "PRIMARY">>

NeverReachTargetHeight == nextHeightToTry < TARGET_HEIGHT

EvidenceWhenFaultyInv ==
    (state[1] = "FoundEvidence") => (~IS_PRIMARY_CORRECT \/ ~IS_SECONDARY_CORRECT)

NoEvidenceForCorrectInv ==
    IS_PRIMARY_CORRECT /\ IS_SECONDARY_CORRECT => evidences = {} <: {ET}

(**
 * If we find an evidence by peer A, peer B has ineded given us a corrupted
 * header following the common height. Also, we have a verification trace by peer A.
 *)
CommonHeightOnEvidenceInv ==
  \A e \in evidences:
    LET conflicting == e.conflictingBlock IN
    LET conflictingHeader == conflicting.header IN
    \* the evidence by suspectingPeer can be verified by suspectingPeer in one step
    LET SoundEvidence(suspectingPeer, peerBlocks) ==
      \/ e.peer /= suspectingPeer
        \* the conflicting block from another peer verifies against the common height
      \/ /\ "SUCCESS" =
            LC!ValidAndVerifiedUntimed(peerBlocks[e.commonHeight], conflicting)
         \* and the headers of the same height by the two peers do not match
         /\ peerBlocks[conflictingHeader.height].header /= conflictingHeader
    IN
    /\ SoundEvidence("PRIMARY", fetchedLightBlocks1b)
    /\ SoundEvidence("SECONDARY", fetchedLightBlocks2)

(**
 * If the light client does not find an evidence,
 * then there is no attack on the light client.
 *)
AccuracyInv ==
  (LC!InTrustingPeriodLocal(fetchedLightBlocks1[TARGET_HEIGHT].header)
        /\ state = <<"NoEvidence", "SECONDARY">>)
        =>
    (fetchedLightBlocks1[TARGET_HEIGHT].header = blockchain[TARGET_HEIGHT]
      /\ fetchedLightBlocks2[TARGET_HEIGHT].header = blockchain[TARGET_HEIGHT])

(**
 * The primary reports a corrupted block at the target height. If the secondary is
 * correct and the algorithm has terminated, we should get the evidence.
 * This property is violated due to clock drift. VerifyToTarget may fail with
 * the correct secondary within the trusting period (due to clock drift, locally
 * we think that we are outside of the trusting period).
 *)
PrecisionInvGrayZone ==
    (/\ fetchedLightBlocks1[TARGET_HEIGHT].header /= blockchain[TARGET_HEIGHT]
     /\ BC!InTrustingPeriod(blockchain[TRUSTED_HEIGHT])
     /\ IS_SECONDARY_CORRECT
     /\ IsTerminated)
       =>
    evidences /= {} <: {ET}

(**
 * The primary reports a corrupted block at the target height. If the secondary is
 * correct and the algorithm has terminated, we should get the evidence.
 * This invariant does not fail, as we are using the local clock to check the trusting
 * period.
 *)
PrecisionInvLocal ==
    (/\ fetchedLightBlocks1[TARGET_HEIGHT].header /= blockchain[TARGET_HEIGHT]
     /\ LC!InTrustingPeriodLocalSurely(blockchain[TRUSTED_HEIGHT])
     /\ IS_SECONDARY_CORRECT
     /\ IsTerminated)
       =>
    evidences /= {} <: {ET}

====================================================================================
