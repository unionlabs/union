# Draft of Light Client Supervisor for discussion

## Modification to the initialization

The lightclient is initialized with LCInitData

### **[LC-DATA-INIT.2]**

```go
type LCInitData struct {
    TrustedBlock   LightBlock
    Genesis        GenesisDoc
    TrustedHash    []byte
    TrustedHeight  int64
}
```

where only one of the components must be provided. `GenesisDoc` is
defined in the [CometBFT
Types](https://github.com/cometbft/cometbft/blob/main/types/genesis.go).


### Initialization

The light client is based on subjective initialization. It has to
trust the initial data given to it by the user. It cannot perform any
detection of an attack yet instead requires an initial point of trust.
There are three forms of initial data which are used to obtain the
first trusted block:

- A trusted block from a prior initialization
- A trusted height and hash
- A genesis file

The golang light client implementation checks this initial data in that
order; first attempting to find a trusted block from the trusted store,
then acquiring a light block from the primary at the trusted height and matching
the hash, or finally checking for a genesis file to verify the initial header.

The light client doesn't need to check if the trusted block is within the
trusted period because it already trusts it, however, if the light block is
outside the trust period, there is a higher chance the light client won't be
able to verify anything.

Cross-checking this trusted block with providers upon initialization is helpful
for ensuring that the node is responsive and correctly configured but does not
increase trust since proving a conflicting block is a
[light client attack](https://github.com/cometbft/cometbft/blob/main/spec/light-client/detection/detection_003_reviewed.md#cmbc-lc-attack1)
and not just a [bogus](https://github.com/cometbft/cometbft/blob/main/spec/light-client/detection/detection_003_reviewed.md#cmbc-bogus1) block could result in
performing backwards verification beyond the trusted period, thus a fruitless
endeavour.

However, with the notion of it's better to fail earlier than later, the golang
light client implementation will perform a consistency check on all providers
and will error if one returns a different header, allowing the user
the opportunity to reinitialize.

#### **[LC-FUNC-INIT.2]:**

```go
func InitLightClient(initData LCInitData) (LightStore, Error) {
    var initialBlock LightBlock

    switch {
    case LCInitData.TrustedBlock != nil:
        // we trust the block from a prior initialization
        initialBlock = LCInitData.TrustedBlock

    case LCInitData.TrustedHash != nil:
        untrustedBlock := FetchLightBlock(PeerList.Primary(), LCInitData.TrustedHeight)
        

        // verify that the hashes match
        if untrustedBlock.Hash() != LCInitData.TrustedHash {
            return nil, Error("Primary returned block with different hash")
        }
        // after checking the hash we now trust the block
        initialBlock = untrustedBlock        
    }
    case LCInitData.Genesis != nil:
        untrustedBlock := FetchLightBlock(PeerList.Primary(), LCInitData.Genesis.InitialHeight)
        
        // verify that 2/3+ of the validator set signed the untrustedBlock
        if err := VerifyCommitFull(untrustedBlock.Commit, LCInitData.Genesis.Validators); err != nil {
            return nil, err
        }

        // we can now trust the block
        initialBlock = untrustedBlock
    default:
        return nil, Error("No initial data was provided")

    // This is done in the golang version but is optional and not strictly part of the protocol
    if err := CrossCheck(initialBlock, PeerList.Witnesses()); err != nil {
        return nil, err
    }

    // initialize light store
    lightStore := new LightStore;
    lightStore.Add(newBlock);
    return (lightStore, OK);
}

func CrossCheck(lb LightBlock, witnesses []Provider) error {
    for _, witness := range witnesses {
        witnessBlock := FetchLightBlock(witness, lb.Height)

        if witnessBlock.Hash() != lb.Hash() {
            return Error("Witness has different block")
        }
    }
    return OK
}

```

- Implementation remark
    - none
- Expected precondition
    - *LCInitData* contains either a genesis file of a lightblock
    - if genesis it passes `ValidateAndComplete()` see [CometBFT](https://informal.systems)
- Expected postcondition
    - *lightStore* initialized with trusted lightblock. It has either been
      cross-checked (from genesis) or it has initial trust from the
      user.
- Error condition
    - if precondition is violated
    - empty peerList

----

