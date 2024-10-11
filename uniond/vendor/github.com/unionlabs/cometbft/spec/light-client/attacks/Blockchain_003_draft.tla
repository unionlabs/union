------------------------ MODULE Blockchain_003_draft -----------------------------
(*
  This is a high-level specification of a Cosmos blockchain
  that is designed specifically for the light client.
  Validators have the voting power of one. If you like to model various
  voting powers, introduce multiple copies of the same validator
  (do not forget to give them unique names though).
 *)
EXTENDS Integers, FiniteSets, Apalache

Min(a, b) == IF a < b THEN a ELSE b

CONSTANT
  AllNodes,
    (* a set of all nodes that can act as validators (correct and faulty) *)
  ULTIMATE_HEIGHT,
    (* a maximal height that can be ever reached (modelling artifact) *)
  TRUSTING_PERIOD
    (* the period within which the validators are trusted *)

Heights == 1..ULTIMATE_HEIGHT   (* possible heights *)

(* A commit is just a set of nodes who have committed the block *)
Commits == SUBSET AllNodes

(* The set of all block headers that can be on the blockchain.
   This is a simplified version of the Block data structure in the actual implementation. *)
BlockHeaders == [
  height: Heights,
    \* the block height
  time: Int,
    \* the block timestamp in some integer units
  lastCommit: Commits,
    \* the nodes who have voted on the previous block, the set itself instead of a hash
  (* in the implementation, only the hashes of V and NextV are stored in a block,
     as V and NextV are stored in the application state *) 
  VS: SUBSET AllNodes,
    \* the validators of this block. We store the validators instead of the hash.
  NextVS: SUBSET AllNodes
    \* the validators of the next block. We store the next validators instead of the hash.
]

(* A signed header is just a header together with a set of commits *)
LightBlocks == [header: BlockHeaders, Commits: Commits]

VARIABLES
    refClock,
        (* the current global time in integer units as perceived by the reference chain *)
    blockchain,
    (* A sequence of BlockHeaders, which gives us a bird view of the blockchain. *)
    Faulty
    (* A set of faulty nodes, which can act as validators. We assume that the set
       of faulty processes is non-decreasing. If a process has recovered, it should
       connect using a different id. *)
       
(* all variables, to be used with UNCHANGED *)       
vars == <<refClock, blockchain, Faulty>>         

(* The set of all correct nodes in a state *)
Corr == AllNodes \ Faulty

(* APALACHE annotations *)
a <: b == a \* type annotation

NT == STRING
NodeSet(S) == S <: {NT}
EmptyNodeSet == NodeSet({})

BT == [height |-> Int, time |-> Int, lastCommit |-> {NT}, VS |-> {NT}, NextVS |-> {NT}]

LBT == [header |-> BT, Commits |-> {NT}]
(* end of APALACHE annotations *)       

(****************************** BLOCKCHAIN ************************************)

(* the header is still within the trusting period *)
InTrustingPeriod(header) ==
    refClock < header.time + TRUSTING_PERIOD

(*
 Given a function pVotingPower \in D -> Powers for some D \subseteq AllNodes
 and pNodes \subseteq D, test whether the set pNodes \subseteq AllNodes has
 more than 2/3 of voting power among the nodes in D.
 *)
TwoThirds(pVS, pNodes) ==
    LET TP == Cardinality(pVS)
        SP == Cardinality(pVS \intersect pNodes)
    IN
    3 * SP > 2 * TP \* when thinking in real numbers, not integers: SP > 2.0 / 3.0 * TP 

(*
 Given a set of FaultyNodes, test whether the voting power of the correct nodes in D
 is more than 2/3 of the voting power of the faulty nodes in D.

 Parameters:
   - pFaultyNodes is a set of nodes that are considered faulty
   - pVS is a set of all validators, maybe including Faulty, intersecting with it, etc.
   - pMaxFaultRatio is a pair <<a, b>> that limits the ratio a / b of the faulty
     validators from above (exclusive)
 *)
FaultyValidatorsFewerThan(pFaultyNodes, pVS, maxRatio) ==
    LET FN == pFaultyNodes \intersect pVS   \* faulty nodes in pNodes
        CN == pVS \ pFaultyNodes            \* correct nodes in pNodes
        CP == Cardinality(CN)               \* power of the correct nodes
        FP == Cardinality(FN)               \* power of the faulty nodes
    IN
    \* CP + FP = TP is the total voting power
    LET TP == CP + FP IN
    FP * maxRatio[2] < TP * maxRatio[1]

(* Can a block be produced by a correct peer, or an authenticated Byzantine peer *)
IsLightBlockAllowedByDigitalSignatures(ht, block) == 
    \/ block.header = blockchain[ht] \* signed by correct and faulty (maybe)
    \/ /\ block.Commits \subseteq Faulty
       /\ block.header.height = ht
       /\ block.header.time >= 0 \* signed only by faulty

(*
 Initialize the blockchain to the ultimate height right in the initial states.
 We pick the faulty validators statically, but that should not affect the light client.

 Parameters:
    - pMaxFaultyRatioExclusive is a pair <<a, b>> that bound the number of
        faulty validators in each block by the ratio a / b (exclusive)
 *)            
InitToHeight(pMaxFaultyRatioExclusive) ==
  /\ \E Nodes \in SUBSET AllNodes:
      Faulty := Nodes   \* pick a subset of nodes to be faulty
  \* pick the validator sets and last commits
  /\ \E vs, lastCommit \in [Heights -> SUBSET AllNodes]:
     \E timestamp \in [Heights -> Int]:
        \* refClock is at least as early as the timestamp in the last block 
        /\ \E tm \in Int:
          refClock := tm /\ tm >= timestamp[ULTIMATE_HEIGHT]
        \* the genesis starts on day 1     
        /\ timestamp[1] = 1
        /\ vs[1] = AllNodes
        /\ lastCommit[1] = EmptyNodeSet
        /\ \A h \in Heights \ {1}:
          /\ lastCommit[h] \subseteq vs[h - 1]   \* the non-validators cannot commit 
          /\ TwoThirds(vs[h - 1], lastCommit[h]) \* the commit has >2/3 of validator votes
            \* the faulty validators have the power below the threshold
          /\ FaultyValidatorsFewerThan(Faulty, vs[h], pMaxFaultyRatioExclusive)
          /\ timestamp[h] > timestamp[h - 1]     \* the time grows monotonically
          /\ timestamp[h] < timestamp[h - 1] + TRUSTING_PERIOD    \* but not too fast
        \* form the block chain out of validator sets and commits (this makes apalache faster)
        /\ blockchain := [h \in Heights |->
             [height |-> h,
              time |-> timestamp[h],
              VS |-> vs[h],
              NextVS |-> IF h < ULTIMATE_HEIGHT THEN vs[h + 1] ELSE AllNodes,
              lastCommit |-> lastCommit[h]]
             ] \******
       
(********************* BLOCKCHAIN ACTIONS ********************************)
(*
  Advance the clock by zero or more time units.
  *)
AdvanceTime ==
  /\ \E tm \in Int: tm >= refClock /\ refClock' = tm
  /\ UNCHANGED <<blockchain, Faulty>>

=============================================================================
\* Modification History
\* Last modified Wed Jun 10 14:10:54 CEST 2020 by igor
\* Created Fri Oct 11 15:45:11 CEST 2019 by igor
