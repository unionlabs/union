# Draft of Functions for Fork detection and Proof of Fork Submisstion

This document collects drafts of function for generating and
submitting proof of fork in the IBC context

- [IBC](#on-chain-ibc-component)

- [Relayer](#relayer)

## On-chain IBC Component

> The following is a suggestions to change the function defined in ICS 007

#### [TAG-IBC-MISBEHAVIOR.1]

```go
func checkMisbehaviorAndUpdateState(cs: ClientState, PoF: LightNodeProofOfFork)
```

**TODO:** finish conditions

- Implementation remark
- Expected precondition
    - PoF.TrustedBlock.Header is equal to lightBlock on store with
      same height
    - both traces end with header of same height
    - headers are different
    - both traces are supported by PoF.TrustedBlock (`supports`
   defined in [CMBC-FUNC]), that is, for `t = currentTimestamp()` (see
   ICS 024)
        - supports(PoF.TrustedBlock, PoF.PrimaryTrace[1], t)
        - supports(PoF.PrimaryTrace[i], PoF.PrimaryTrace[i+1], t) for
     *0 < i < length(PoF.PrimaryTrace)*
        - supports(PoF.TrustedBlock,  PoF.SecondaryTrace[1], t)
        - supports(PoF.SecondaryTrace[i], PoF.SecondaryTrace[i+1], t) for
     *0 < i < length(PoF.SecondaryTrace)*
- Expected postcondition
    - set cs.FrozenHeight to min(cs.FrozenHeight, PoF.TrustedBlock.Header.Height)
- Error condition
    - none

----

> The following is a suggestions to add functionality to ICS 002 and 007.
> I suppose the above is the most efficient way to get the required
> information. Another option is to subscribe to "header install"
> events via CosmosSDK

#### [TAG-IBC-HEIGHTS.1]

```go
func QueryHeightsRange(id, from, to) ([]Height)
```

- Expected postcondition
    - returns all heights *h*, with *from <= h <= to* for which the
      IBC component has a consensus state.

----

> This function can be used if the relayer has no information about
> the IBC component. This allows late-joining relayers to also
> participate in fork detection and the generation in proof of
> fork. Alternatively, we may also postulate that relayers are not
> responsible to detect forks for heights before they started (and
> subscribed to the transactions reporting fresh headers being
> installed at the IBC component).

## Relayer

### Auxiliary Functions to be implemented in the Light Client

#### [LCV-LS-FUNC-GET-PREV.1]

```go
func (ls LightStore) GetPreviousVerified(height Height) (LightBlock, bool)
```

- Expected postcondition
    - returns a verified LightBlock, whose height is maximal among all
      verified lightblocks with height smaller than `height`

----

### Relayer Submitting Proof of Fork to the IBC Component

There are two ways the relayer can detect a fork

- by the fork detector of one of its lightclients
- be checking the consensus state of the IBC component

The following function ignores how the proof of fork was generated.
It takes a proof of fork as input and computes a proof of fork that
     will be accepted by the IBC component.
The problem addressed here is that both, the relayer's light client
     and the IBC component have incomplete light stores, that might
     not have all light blocks in common.
Hence the relayer has to figure out what the IBC component knows
     (intuitively, a meeting point between the two lightstores
     computed in `commonRoot`)  and compute a proof of fork
     (`extendPoF`) that the IBC component will accept based on its
     knowledge.

The auxiliary functions `commonRoot` and `extendPoF` are
defined below.

#### [TAG-SUBMIT-POF-IBC.1]

```go
func SubmitIBCProofOfFork(
  lightStore LightStore,
  PoF: LightNodeProofOfFork,
  ibc IBCComponent) (Error) {
    if ibc.queryChainConsensusState(PoF.TrustedBlock.Height) = PoF.TrustedBlock {
  // IBC component has root of PoF on store, we can just submit
        ibc.submitMisbehaviorToClient(ibc.id,PoF)
  return Success
     // note sure about the id parameter
    }
    else {
        // the ibc component does not have the TrustedBlock and might
  // even be on yet a different branch. We have to compute a PoF
  // that the ibc component can verify based on its current
        // knowledge

        ibcLightBlock, lblock, _, result := commonRoot(lightStore, ibc, PoF.TrustedBlock)

     if result = Success {
   newPoF = extendPoF(ibcLightBlock, lblock, lightStore, PoF)
      ibc.submitMisbehaviorToClient(ibc.id, newPoF)
      return Success
     }
  else{
   return CouldNotGeneratePoF
     }
    }
}
```

**TODO:** finish conditions

- Implementation remark
- Expected precondition
- Expected postcondition
- Error condition
    - none

----

### Auxiliary Functions at the Relayer

> If the relayer detects a fork, it has to compute a proof of fork that
> will convince the IBC component. That is it has to compare the
> relayer's local lightstore against the lightstore of the IBC
> component, and find common ancestor lightblocks.

#### [TAG-COMMON-ROOT.1]

```go
func commonRoot(lightStore LightStore, ibc IBCComponent, lblock
LightBlock) (LightBlock, LightBlock, LightStore, Result) {

    auxLS.Init

       // first we ask for the heights the ibc component is aware of
  ibcHeights = ibc.QueryHeightsRange(
                     ibc.id,
                     lightStore.LowestVerified().Height,
                     lblock.Height - 1);
  // this function does not exist yet. Alternatively, we may
  // request all transactions that installed headers via CosmosSDK


        for {
            h, result = max(ibcHeights)
   if result = Empty {
       return (_, _, _, NoRoot)
      }
      ibcLightBlock = ibc.queryChainConsensusState(h)
   auxLS.Update(ibcLightBlock, StateVerified);
      connector, result := Connector(lightStore, ibcLightBlock, lblock.Header.Height)
      if result = success {
       return (ibcLightBlock, connector, auxLS, Success)
   }
   else{
       ibcHeights.remove(h)
      }
  }
}
```

- Expected postcondition
    - returns
        - a lightBlock b1 from the IBC component, and
        - a lightBlock b2
            from the local lightStore with height less than
   lblock.Header.Height, s.t. b1 supports b2, and
        - a lightstore with the blocks downloaded from
          the ibc component

----

#### [TAG-LS-FUNC-CONNECT.1]

```go
func Connector (lightStore LightStore, lb LightBlock, h Height) (LightBlock, bool)
```

- Expected postcondition
    - returns a verified LightBlock from lightStore with height less
      than *h* that can be
      verified by lb in one step.

**TODO:** for the above to work we need an invariant that all verified
lightblocks form a chain of trust. Otherwise, we need a lightblock
that has a chain of trust to height.

> Once the common root is found, a proof of fork that will be accepted
> by the IBC component needs to be generated. This is done in the
> following function.

#### [TAG-EXTEND-POF.1]

```go
func extendPoF (root LightBlock,
                connector LightBlock,
    lightStore LightStore,
    Pof LightNodeProofofFork) (LightNodeProofofFork}
```

- Implementation remark
    - PoF is not sufficient to convince an IBC component, so we extend
      the proof of fork farther in the past
- Expected postcondition
    - returns a newPOF:
        - newPoF.TrustedBlock = root
        - let prefix =
       connector +
       lightStore.Subtrace(connector.Header.Height, PoF.TrustedBlock.Header.Height-1) +
       PoF.TrustedBlock
            - newPoF.PrimaryTrace = prefix + PoF.PrimaryTrace
            - newPoF.SecondaryTrace = prefix + PoF.SecondaryTrace

### Detection a fork at the IBC component

The following functions is assumed to be called regularly to check
that latest consensus state of the IBC component. Alternatively, this
logic can be executed whenever the relayer is informed (via an event)
that a new header has been installed.

#### [TAG-HANDLER-DETECT-FORK.1]

```go
func DetectIBCFork(ibc IBCComponent, lightStore LightStore) (LightNodeProofOfFork, Error) {
    cs = ibc.queryClientState(ibc);
 lb, found := lightStore.Get(cs.Header.Height)
    if !found {
 **TODO:** need verify to target
        lb, result = LightClient.Main(primary, lightStore, cs.Header.Height)
  // [LCV-FUNC-IBCMAIN.1]
  **TODO** decide what to do following the outcome of Issue #499

  // I guess here we have to get into the light client

    }
 if cs != lb {
     // IBC component disagrees with my primary.
  // I fetch the
     ibcLightBlock, lblock, ibcStore, result := commonRoot(lightStore, ibc, lb)
  pof = new LightNodeProofOfFork;
  pof.TrustedBlock := ibcLightBlock
  pof.PrimaryTrace := ibcStore + cs
  pof.SecondaryTrace :=  lightStore.Subtrace(lblock.Header.Height,
                                    lb.Header.Height);
        return(pof, Fork)
 }
 return(nil , NoFork)
}
```

**TODO:** finish conditions

- Implementation remark
    - we ask the handler for the latest check. Cross-check with the
      chain. In case they deviate we generate PoF.
    - we assume IBC component is correct. It has verified the
      consensus state
- Expected precondition
- Expected postcondition
