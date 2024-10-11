-------------------- MODULE LCVerificationApi_003_draft --------------------------
(**
 * The common interface of the light client verification and detection.
 *) 
EXTENDS Integers, FiniteSets

\* the parameters of Light Client
CONSTANTS
  TRUSTING_PERIOD,
    (* the period within which the validators are trusted *)
  CLOCK_DRIFT,
    (* the assumed precision of the clock *)
  REAL_CLOCK_DRIFT,
    (* the actual clock drift, which under normal circumstances should not
       be larger than CLOCK_DRIFT (otherwise, there will be a bug) *)
  FAULTY_RATIO
    (* a pair <<a, b>> that limits that ratio of faulty validator in the blockchain
       from above (exclusive). Cosmos security model prescribes 1 / 3. *)

VARIABLES  
  localClock (* current time as measured by the light client *)

(* the header is still within the trusting period *)
InTrustingPeriodLocal(header) ==
    \* note that the assumption about the drift reduces the period of trust
    localClock < header.time + TRUSTING_PERIOD - CLOCK_DRIFT

(* the header is still within the trusting period, even if the clock can go backwards *)
InTrustingPeriodLocalSurely(header) ==
    \* note that the assumption about the drift reduces the period of trust
    localClock < header.time + TRUSTING_PERIOD - 2 * CLOCK_DRIFT

(* ensure that the local clock does not drift far away from the global clock *)
IsLocalClockWithinDrift(local, global) ==
    /\ global - REAL_CLOCK_DRIFT <= local
    /\ local <= global + REAL_CLOCK_DRIFT

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
 The first part of the precondition of ValidAndVerified, which does not take
 the current time into account.
 
 [LCV-FUNC-VALID.1::TLA-PRE-UNTIMED.1]
 *)
ValidAndVerifiedPreUntimed(trusted, untrusted) ==
  LET thdr == trusted.header
      uhdr == untrusted.header
  IN
  /\ thdr.height < uhdr.height
     \* the trusted block has been created earlier
  /\ thdr.time < uhdr.time
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
 Check the precondition of ValidAndVerified, including the time checks.
 
 [LCV-FUNC-VALID.1::TLA-PRE.1]
 *)
ValidAndVerifiedPre(trusted, untrusted, checkFuture) ==
  LET thdr == trusted.header
      uhdr == untrusted.header
  IN
  /\ InTrustingPeriodLocal(thdr)
     \* The untrusted block is not from the future (modulo clock drift).
     \* Do the check, if it is required.
  /\ checkFuture => uhdr.time < localClock + CLOCK_DRIFT
  /\ ValidAndVerifiedPreUntimed(trusted, untrusted)


(**
 Check, whether an untrusted block is valid and verifiable w.r.t. a trusted header.
 This test does take current time into account, but only looks at the block structure.
 
 [LCV-FUNC-VALID.1::TLA-UNTIMED.1]
 *)   
ValidAndVerifiedUntimed(trusted, untrusted) ==
    IF ~ValidAndVerifiedPreUntimed(trusted, untrusted)
    THEN "INVALID"
    ELSE IF untrusted.header.height = trusted.header.height + 1
             \/ SignedByOneThirdOfTrusted(trusted, untrusted)
         THEN "SUCCESS"
         ELSE "NOT_ENOUGH_TRUST"

(**
 Check, whether an untrusted block is valid and verifiable w.r.t. a trusted header.
 
 [LCV-FUNC-VALID.1::TLA.1]
 *)   
ValidAndVerified(trusted, untrusted, checkFuture) ==
    IF ~ValidAndVerifiedPre(trusted, untrusted, checkFuture)
    THEN "INVALID"
    ELSE IF ~InTrustingPeriodLocal(untrusted.header)
    (* We leave the following test for the documentation purposes.
       The implementation should do this test, as signature verification may be slow.
       In the TLA+ specification, ValidAndVerified happens in no time.
     *) 
    THEN "FAILED_TRUSTING_PERIOD" 
    ELSE IF untrusted.header.height = trusted.header.height + 1
             \/ SignedByOneThirdOfTrusted(trusted, untrusted)
         THEN "SUCCESS"
         ELSE "NOT_ENOUGH_TRUST"


(**
  The invariant of the light store that is not related to the blockchain
 *)
LightStoreInv(fetchedLightBlocks, lightBlockStatus) ==
    \A lh, rh \in DOMAIN fetchedLightBlocks:
            \* for every pair of stored headers that have been verified
        \/ lh >= rh
        \/ lightBlockStatus[lh] /= "StateVerified"
        \/ lightBlockStatus[rh] /= "StateVerified"
           \* either there is a header between them
        \/ \E mh \in DOMAIN fetchedLightBlocks:
            lh < mh /\ mh < rh /\ lightBlockStatus[mh] = "StateVerified"
           \* or the left header is outside the trusting period, so no guarantees
        \/ LET lhdr == fetchedLightBlocks[lh]
               rhdr == fetchedLightBlocks[rh]
           IN
           \* we can verify the right one using the left one
           "SUCCESS" = ValidAndVerifiedUntimed(lhdr, rhdr)

(**
  Correctness states that all the obtained headers are exactly like in the blockchain.
 
  It is always the case that every verified header in LightStore was generated by
  an instance of Tendermint consensus.
  
  [LCV-DIST-SAFE.1::CORRECTNESS-INV.1]
 *)  
CorrectnessInv(blockchain, fetchedLightBlocks, lightBlockStatus) ==
    \A h \in DOMAIN fetchedLightBlocks:
        lightBlockStatus[h] = "StateVerified" =>
            fetchedLightBlocks[h].header = blockchain[h]

(**
 * When the light client terminates, there are no failed blocks.
 * (Otherwise, someone lied to us.) 
 *)            
NoFailedBlocksOnSuccessInv(fetchedLightBlocks, lightBlockStatus) ==
     \A h \in DOMAIN fetchedLightBlocks:
        lightBlockStatus[h] /= "StateFailed"            

(**
 The expected post-condition of VerifyToTarget.
 *)
VerifyToTargetPost(blockchain, isPeerCorrect,
                   fetchedLightBlocks, lightBlockStatus,
                   trustedHeight, targetHeight, finalState) ==
  LET trustedHeader == fetchedLightBlocks[trustedHeight].header IN
    \* The light client is not lying us on the trusted block.
    \* It is straightforward to detect.
    /\ lightBlockStatus[trustedHeight] = "StateVerified"
    /\ trustedHeight \in DOMAIN fetchedLightBlocks
    /\ trustedHeader = blockchain[trustedHeight]
    \* the invariants we have found in the light client verification
    \* there is a problem with trusting period
    /\ isPeerCorrect
        => CorrectnessInv(blockchain, fetchedLightBlocks, lightBlockStatus)
    \* a correct peer should fail the light client,
    \* if the trusted block is in the trusting period
    /\ isPeerCorrect /\ InTrustingPeriodLocalSurely(trustedHeader)
        => finalState = "finishedSuccess"
    /\ finalState = "finishedSuccess" =>
        /\ lightBlockStatus[targetHeight] = "StateVerified"
        /\ targetHeight \in DOMAIN fetchedLightBlocks
        /\ NoFailedBlocksOnSuccessInv(fetchedLightBlocks, lightBlockStatus)
    /\ LightStoreInv(fetchedLightBlocks, lightBlockStatus)


==================================================================================
