# Results of Discussions and Decisions

- Generating a minimal proof of fork (as suggested in [Issue #5083](https://github.com/tendermint/tendermint/issues/5083)) is too costly at the light client
    - we do not know all lightblocks from the primary
    - therefore there are many scenarios. we might even need to ask
      the primary again for additional lightblocks to isolate the
      branch.

> For instance, the light node starts with block at height 1 and the
> primary provides a block of height 10 that the light node can
> verify immediately. In cross-checking, a secondary now provides a
> conflicting header b10 of height 10 that needs another header b5
> of height 5 to
> verify. Now, in order for the light node to convince the primary:
>
> - The light node cannot just sent b5, as it is not clear whether
>     the fork happened before or after 5
> - The light node cannot just send b10, as the primary would also
>     need  b5 for verification
> - In order to minimize the evidence, the light node may try to
>     figure out where the branch happens, e.g., by asking the primary
>     for height 5 (it might be that more queries are required, also
>     to the secondary. However, assuming that in this scenario the
>     primary is faulty it may not respond.

   As the main goal is to catch misbehavior of the primary,
      evidence generation and punishment must not depend on their
      cooperation. So the moment we have proof of fork (even if it
      contains several light blocks) we should submit right away.

- decision: "full" proof of fork consists of two traces that originate in the
  same lightblock and lead to conflicting headers of the same height.
  
- For submission of proof of fork, we may do some optimizations, for
  instance, we might just submit  a trace of lightblocks that verifies a block
  different from the one the full node knows (we do not send the trace
  the primary gave us back to the primary)

- The light client attack is via the primary. Thus we try to
  catch if the primary installs a bad light block
    - We do not check secondary against secondary
    - For each secondary, we check the primary against one secondary

- Observe that just two blocks for the same height are not
sufficient proof of fork.
One of the blocks may be bogus [CMBC-BOGUS.1] which does
not constitute slashable behavior.  
Which leads to the question whether the light node should try to do
fork detection on its initial block (from subjective
initialization). This could be done by doing backwards verification
(with the hashes) until a bifurcation block is found.
While there are scenarios where a
fork could be found, there is also the scenario where a faulty full
node feeds the light node with bogus light blocks and forces the light
node to check hashes until a bogus chain is out of the trusting period.
As a result, the light client
should not try to detect a fork for its initial header. **The initial
header must be trusted as is.**

# Light Client Sequential Supervisor

**TODO:** decide where (into which specification) to put the
following:

We describe the context on which the fork detector is called by giving
a sequential version of the supervisor function.
Roughly, it alternates two phases namely:

- Light Client Verification. As a result, a header of the required
     height has been downloaded from and verified with the primary.
- Light Client Fork Detections. As a result the header has been
     cross-checked with the secondaries. In case there is a fork we
     submit "proof of fork" and exit.

#### **[LC-FUNC-SUPERVISOR.1]:**

```go
func Sequential-Supervisor () (Error) {
    loop {
     // get the next height
        nextHeight := input();
  
  // Verify
        result := NoResult;
        while result != ResultSuccess {
            lightStore,result := VerifyToTarget(primary, lightStore, nextHeight);
            if result == ResultFailure {
    // pick new primary (promote a secondary to primary)
    /// and delete all lightblocks above
             // LastTrusted (they have not been cross-checked)
             Replace_Primary();
   }
        }
  
  // Cross-check
        PoFs := Forkdetector(lightStore, PoFs);
        if PoFs.Empty {
      // no fork detected with secondaries, we trust the new
   // lightblock
            LightStore.Update(testedLB, StateTrusted);
        }
        else {
      // there is a fork, we submit the proofs and exit
            for i, p range PoFs {
                SubmitProofOfFork(p);
            }
            return(ErrorFork);
        }
    }
}
```

**TODO:** finish conditions

- Implementation remark
- Expected precondition
    - *lightStore* initialized with trusted header
    - *PoFs* empty
- Expected postcondition
    - runs forever, or
    - is terminated by user and satisfies LightStore invariant, or **TODO**
    - has submitted proof of fork upon detecting a fork
- Error condition
    - none

----

# Semantics of the LightStore

Currently, a lightblock in the lightstore can be in one of the
following states:

- StateUnverified
- StateVerified
- StateFailed
- StateTrusted

The intuition is that `StateVerified` captures that the lightblock has
been verified with the primary, and `StateTrusted` is the state after
successful cross-checking with the secondaries.

Assuming there is **always one correct node among primary and
secondaries**, and there is no fork on the blockchain, lightblocks that
are in `StateTrusted` can be used by the user with the guarantee of
"finality". If a block in  `StateVerified` is used, it might be that
detection later finds a fork, and a roll-back might be needed.

**Remark:** The assumption of one correct node, does not render
verification useless. It is true that if the primary and the
secondaries return the same block we may trust it. However, if there
is a node that provides a different block, the light node still needs
verification to understand whether there is a fork, or whether the
different block is just bogus (without any support of some previous
validator set).

**Remark:** A light node may choose the full nodes it communicates
with (the light node and the full node might even belong to the same
stakeholder) so the assumption might be justified in some cases.

In the future, we will do the following changes

- we assume that only from time to time, the light node is
     connected to a correct full node
- this means for some limited time, the light node might have no
     means to defend against light client attacks
- as a result we do not have finality
- once the light node reconnects with a correct full node, it
     should detect the light client attack and submit evidence.

Under these assumptions, `StateTrusted` loses its meaning. As a
result, it should be removed from the API. We suggest that we replace
it with a flag "trusted" that can be used

- internally for efficiency reasons (to maintain
  [LCD-INV-TRUSTED-AGREED.1] until a fork is detected)
- by light client based on the "one correct full node" assumption

----
