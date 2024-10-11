------------------------- MODULE supervisor_001_draft ------------------------
(*
This is the beginning of a spec that will eventually use verification and detector API
*)

EXTENDS Integers, FiniteSets

VARIABLES
    state,
    output

vars == <<state, output>>

CONSTANT
    INITDATA
    
Init ==
    /\ state = "Init"
    /\ output = "none"
    
NextInit ==
    /\ state = "Init"
    /\ \/ state' = "EnterLoop"
       \/ state' = "FailedToInitialize"
    /\ UNCHANGED output
       
NextVerifyToTarget ==
    /\ state = "EnterLoop"
    /\ \/ state' = "EnterLoop" \* replace primary
       \/ state' = "EnterDetect"
       \/ state' = "ExhaustedPeersPrimary"
    /\ UNCHANGED output

NextAttackDetector ==
    /\ state = "EnterDetect"
    /\ \/ state' = "NoEvidence"
       \/ state' = "EvidenceFound"
       \/ state' = "ExhaustedPeersSecondaries"      
    /\ UNCHANGED output

NextVerifyAndDetect ==
    \/ NextVerifyToTarget
    \/ NextAttackDetector
    
NextOutput ==
    /\ state = "NoEvidence"
    /\ state' = "EnterLoop"
    /\ output' = "data" \* to generate a trace
    
NextTerminated ==
    /\ \/ state = "FailedToInitialize"
       \/ state = "ExhaustedPeersPrimary"
       \/ state = "EvidenceFound"
       \/ state = "ExhaustedPeersSecondaries"
    /\ UNCHANGED vars
      
Next ==
    \/ NextInit
    \/ NextVerifyAndDetect
    \/ NextOutput
    \/ NextTerminated

InvEnoughPeers ==
    /\ state /= "ExhaustedPeersPrimary"
    /\ state /= "ExhaustedPeersSecondaries"    
   

=============================================================================
\* Modification History
\* Last modified Sun Oct 18 11:48:45 CEST 2020 by widder
\* Created Sun Oct 18 11:18:53 CEST 2020 by widder
