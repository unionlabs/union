# Light Client Verification

#### **[LCV-FUNC-VERIFYCOMMITLIGHT.1]**

VerifyCommitLight verifies that 2/3+ of the signatures for a validator set were for
a given blockID. The function will finish early and thus may not check all signatures.

```go
func VerifyCommitLight(chainID string, vals *ValidatorSet, blockID BlockID,
height int64, commit *Commit) error {
  // run a basic validation of the arguments
  if err := verifyBasicValsAndCommit(vals, commit, height, blockID); err != nil {
    return err
  }

  // calculate voting power needed
  votingPowerNeeded := vals.TotalVotingPower() * 2 / 3

  var (
    val                *Validator
    valIdx             int32
    seenVals                 = make(map[int32]int, len(commit.Signatures))
    talliedVotingPower int64 = 0
    voteSignBytes      []byte
  )
  for idx, commitSig := range commit.Signatures {
    // ignore all commit signatures that are not for the block
    if !commitSig.ForBlock() {
      continue
    }

    // If the vals and commit have a 1-to-1 correspondence we can retrieve
    // them by index else we need to retrieve them by address
    if lookUpByIndex {
      val = vals.Validators[idx]
    } else {
      valIdx, val = vals.GetByAddress(commitSig.ValidatorAddress)  

      // if the signature doesn't belong to anyone in the validator set
      // then we just skip over it
      if val == nil {
        continue
      }

      // because we are getting validators by address we need to make sure
      // that the same validator doesn't commit twice
      if firstIndex, ok := seenVals[valIdx]; ok {
        secondIndex := idx
        return fmt.Errorf("double vote from %v (%d and %d)", val, firstIndex, secondIndex)
      }
      seenVals[valIdx] = idx
    }

    voteSignBytes = commit.VoteSignBytes(chainID, int32(idx))

    if !val.PubKey.VerifySignature(voteSignBytes, commitSig.Signature) {
      return fmt.Errorf("wrong signature (#%d): %X", idx, commitSig.Signature)
    }

    // Add the voting power of the validator
    // to the tally
    talliedVotingPower += val.VotingPower

    // check if we have enough signatures and can thus exit early
    if talliedVotingPower > votingPowerNeeded {
      return nil
    }
  }

  if got, needed := talliedVotingPower, votingPowerNeeded; got <= needed {
    return ErrNotEnoughVotingPowerSigned{Got: got, Needed: needed}
  }

  return nil
}
```
