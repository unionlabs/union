```
package "github.com/cometbft/cometbft/test/e2e/pkg/grammar/grammar-auto"

Start : CleanStart | Recovery;

CleanStart : InitChain ConsensusExec | StateSync ConsensusExec ;
StateSync : StateSyncAttempts SuccessSync |  SuccessSync ; 
StateSyncAttempts : StateSyncAttempt | StateSyncAttempt StateSyncAttempts ;
StateSyncAttempt : OfferSnapshot ApplyChunks | OfferSnapshot ;
SuccessSync : OfferSnapshot ApplyChunks ; 
ApplyChunks : ApplyChunk | ApplyChunk ApplyChunks ;  

Recovery :  InitChain ConsensusExec | ConsensusExec ;

ConsensusExec : ConsensusHeights ;
ConsensusHeights : ConsensusHeight | ConsensusHeight ConsensusHeights ;
ConsensusHeight : ConsensusRounds FinalizeBlock Commit | FinalizeBlock Commit ;
ConsensusRounds : ConsensusRound | ConsensusRound ConsensusRounds ;
ConsensusRound : Proposer | NonProposer ; 

Proposer : GotVotes | ProposerSimple | Extend | GotVotes ProposerSimple | GotVotes Extend | ProposerSimple Extend | GotVotes ProposerSimple Extend ; 
ProposerSimple : PrepareProposal | PrepareProposal ProcessProposal ;
NonProposer: GotVotes | ProcessProposal | Extend | GotVotes ProcessProposal | GotVotes Extend | ProcessProposal Extend | GotVotes ProcessProposal Extend ; 
Extend : ExtendVote | GotVotes ExtendVote | ExtendVote GotVotes | GotVotes ExtendVote GotVotes ;
GotVotes : GotVote | GotVote GotVotes ; 

InitChain : "init_chain" ;
FinalizeBlock : "finalize_block" ; 
Commit : "commit" ;
OfferSnapshot : "offer_snapshot" ;
ApplyChunk : "apply_snapshot_chunk" ; 
PrepareProposal : "prepare_proposal" ; 
ProcessProposal : "process_proposal" ;
ExtendVote : "extend_vote" ;
GotVote : "verify_vote_extension" ;

```

The original grammar (https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_comet_expected_behavior.md) the grammar above 
refers to is below: 

start               = clean-start / recovery

clean-start         = ( app-handshake / state-sync ) consensus-exec
app-handshake       = info init-chain
state-sync          = *state-sync-attempt success-sync info
state-sync-attempt  = offer-snapshot *apply-chunk
success-sync        = offer-snapshot 1*apply-chunk

recovery            = info [init-chain] consensus-exec

consensus-exec      = (inf)consensus-height
consensus-height    = *consensus-round finalize-block commit
consensus-round     = proposer / non-proposer

proposer            = *got-vote [prepare-proposal [process-proposal]] [extend]
extend              = *got-vote extend-vote *got-vote
non-proposer        = *got-vote [process-proposal] [extend]

init-chain          = %s"<InitChain>"
offer-snapshot      = %s"<OfferSnapshot>"
apply-chunk         = %s"<ApplySnapshotChunk>"
info                = %s"<Info>"
prepare-proposal    = %s"<PrepareProposal>"
process-proposal    = %s"<ProcessProposal>"
extend-vote         = %s"<ExtendVote>"
got-vote            = %s"<VerifyVoteExtension>"
finalize-block      = %s"<FinalizeBlock>"
commit              = %s"<Commit>"

*Note* We ignore `Info` since it can be triggered by the e2e tests at unpredictable places because of its role in RPC handling from external clients. 




