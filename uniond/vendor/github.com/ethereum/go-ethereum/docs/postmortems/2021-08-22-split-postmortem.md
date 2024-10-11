# Minority split 2021-08-27 post mortem

This is a post-mortem concerning the minority split that occurred on Ethereum mainnet on block [13107518](https://etherscan.io/block/13107518), at which a minority chain split occurred.

## Timeline


- 2021-08-17: Guido Vranken submitted a bounty report. Investigation started, root cause identified, patch variations discussed. 
- 2021-08-18: Made public announcement over twitter about upcoming security release upcoming Tuesday. Downstream projects were also notified about the upcoming patch-release.
- 2021-08-24: Released [v1.10.8](https://github.com/ethereum/go-ethereum/releases/tag/v1.10.8) containing the fix on Tuesday morning (CET). Erigon released [v2021.08.04](https://github.com/ledgerwatch/erigon/releases/tag/v2021.08.04).
- 2021-08-27: At 12:50:07 UTC, issue exploited. Analysis started roughly 30m later, 



## Bounty report

###  2021-08-17 RETURNDATA corruption via datacopy

On 2021-08-17, Guido Vranken submitted a report to bounty@ethereum.org. This coincided with a geth-meetup in Berlin, so the geth team could fairly quickly analyse the issue. 

He submitted a proof of concept which called the `dataCopy` precompile, where the input slice and output slice were overlapping but shifted. Doing a `copy` where the `src` and `dest` overlaps is not a problem in itself, however, the `returnData`slice was _also_ using the same memory as a backing-array.

#### Technical details

During CALL-variants, `geth` does not copy the input. This was changed at one point, to avoid a DoS attack reported by Hubert Ritzdorf, to avoid copying data a lot on repeated `CALL`s -- essentially combating a DoS via `malloc`. Further, the datacopy precompile also does not copy the data, but just returns the same slice. This is fine so far. 

After the execution of `dataCopy`, we copy the `ret` into the designated memory area, and this is what causes a problem. Because we're copying a slice of memory over a slice of memory, and this operation modifies (shifts) the data in the source -- the `ret`. So this means we wind up with corrupted returndata.


```
1. Calling datacopy

  memory: [0, 1, 2, 3, 4]
  in (mem[0:4]) : [0,1,2,3]
  out (mem[1:5]): [1,2,3,4]

2. dataCopy returns

  returndata (==in, mem[0:4]): [0,1,2,3]
 
3. Copy in -> out

  => memory: [0,0,1,2,3]
  => returndata: [0,0,1,2]
```


#### Summary

A memory-corruption bug within the EVM can cause a consensus error, where vulnerable nodes obtain a different `stateRoot` when processing a maliciously crafted transaction. This, in turn, would lead to the chain being split: mainnet splitting in two forks.

#### Handling

On the evening of 17th, we discussed options on how to handle it. We made a state test to reproduce the issue, and verified that neither `openethereum`, `nethermind` nor `besu` were affected by the same vulnerability, and started a full-sync with a patched version of `geth`. 

It was decided that in this specific instance, it would be possible to make a public announcement and a patch release: 

- The fix can be made pretty 'generically', e.g. always copying data on input to precompiles. 
- The flaw is pretty difficult to find, given a generic fix in the call. The attacker needs to figure out that it concerns the precompiles, specifically the datcopy, and that it concerns the `RETURNDATA` buffer rather than the regular memory, and lastly the special circumstances to trigger it (overlapping but shifted input/output). 

Since we had merged the removal of `ETH65`, if the entire network were to upgrade, then nodes which have not yet implemented `ETH66` would be cut off from the network. After further discussions, we decided to:

- Announce an upcoming security release on Tuesday (August 24th), via Twitter and official channels, plus reach out to downstream projects.
- Temporarily revert the `ETH65`-removal.
- Place the fix into the PR optimizing the jumpdest analysis [233381](https://github.com/ethereum/go-ethereum/pull/23381). 
- After 4-8 weeks, release details about the vulnerability. 


## Exploit

At block [13107518](https://etherscan.io/block/13107518), mined at Aug-27-2021 12:50:07 PM +UTC, a minority chain split occurred. The discord user @AlexSSD7 notified the allcoredevs-channel on the Eth R&D discord, on Aug 27 13:09  UTC. 


At 14:09 UTC, it was confirmed that the transaction `0x1cb6fb36633d270edefc04d048145b4298e67b8aa82a9e5ec4aa1435dd770ce4` had triggered the bug, leading to a minority-split of the chain. The term 'minority split' means that the majority of miners continued to mine on the correct chain.

At 14:17 UTC, @mhswende tweeted out about the issue [2]. 

The attack was sent from an account funded from Tornado cash. 

It was also found that the same attack had been carried out on the BSC chain at roughly the same time -- at a block mined [12 minutes earlier](https://bscscan.com/tx/0xf667f820631f6adbd04a4c92274374034a3e41fa9057dc42cb4e787535136dce), at Aug-27-2021 12:38:30 PM +UTC. 

The blocks on the 'bad' chain were investigated, and Tim Beiko reached out to those mining operators on the minority chain who could be identified via block extradata. 


## Lessons learned


### Disclosure decision

The geth-team have an official policy regarding [vulnerability disclosure](https://geth.ethereum.org/docs/vulnerabilities/vulnerabilities). 

> The primary goal for the Geth team is the health of the Ethereum network as a whole, and the decision whether or not to publish details about a serious vulnerability boils down to minimizing the risk and/or impact of discovery and exploitation.

In this case, it was decided that public pre-announce + patch would likely lead to sufficient update-window for a critical mass of nodes/miners to upgrade in time before it could be exploited. In hindsight, this was a dangerous decision, and it's unlikely that the same decision would be reached were a similar incident to happen again. 


### Disclosure path

Several subprojects were informed about the upcoming security patch:

- Polygon/Matic
- MEV
- Avalanche
- Erigon
- BSC 
- EWF
- Quorum
- ETC
- xDAI

However, some were 'lost', and only notified later

- Optimism
- Summa
- Harmony

Action point: create a low-volume geth-announce@ethereum.org email list where dependent projects/operators can receive public announcements. 
- This has been done. If you wish to receive release- and security announcements, sign up [here](https://groups.google.com/a/ethereum.org/g/geth-announce/about)

### Fork monitoring

The fork monitor behaved 'ok' during the incident, but had to be restarted during the evening. 

Action point: improve the resiliency of the forkmon, which is currently not performing great when many nodes are connected. 

Action point: enable push-based alerts to be sent from the forkmon, to speed up the fork detection.


## Links

- [1] https://twitter.com/go_ethereum/status/1428051458763763721
- [2] https://twitter.com/mhswende/status/1431259601530458112


## Appendix

### Subprojects


The projects were sent variations of the following text: 
```
We have identified a security issue with go-ethereum, and will issue a
new release (v1.10.8) on Tuesday next week.

At this point, we will not disclose details about the issue, but
recommend downstream/dependent projects to be ready to take actions to
upgrade to the latest go-ethereum codebase. More information about the
issue will be disclosed at a later date.

https://twitter.com/go_ethereum/status/1428051458763763721

```
### Patch

```diff
diff --git a/core/vm/instructions.go b/core/vm/instructions.go
index f7ef2f900e..6c8c6e6e6f 100644
--- a/core/vm/instructions.go
+++ b/core/vm/instructions.go
@@ -669,6 +669,7 @@ func opCall(pc *uint64, interpreter *EVMInterpreter, scope *ScopeContext) ([]byt
        }
        stack.push(&temp)
        if err == nil || err == ErrExecutionReverted {
+               ret = common.CopyBytes(ret)
                scope.Memory.Set(retOffset.Uint64(), retSize.Uint64(), ret)
        }
        scope.Contract.Gas += returnGas
@@ -703,6 +704,7 @@ func opCallCode(pc *uint64, interpreter *EVMInterpreter, scope *ScopeContext) ([
        }
        stack.push(&temp)
        if err == nil || err == ErrExecutionReverted {
+               ret = common.CopyBytes(ret)
                scope.Memory.Set(retOffset.Uint64(), retSize.Uint64(), ret)
        }
        scope.Contract.Gas += returnGas
@@ -730,6 +732,7 @@ func opDelegateCall(pc *uint64, interpreter *EVMInterpreter, scope *ScopeContext
        }
        stack.push(&temp)
        if err == nil || err == ErrExecutionReverted {
+               ret = common.CopyBytes(ret)
                scope.Memory.Set(retOffset.Uint64(), retSize.Uint64(), ret)
        }
        scope.Contract.Gas += returnGas
@@ -757,6 +760,7 @@ func opStaticCall(pc *uint64, interpreter *EVMInterpreter, scope *ScopeContext)
        }
        stack.push(&temp)
        if err == nil || err == ErrExecutionReverted {
+               ret = common.CopyBytes(ret)
                scope.Memory.Set(retOffset.Uint64(), retSize.Uint64(), ret)
        }
        scope.Contract.Gas += returnGas
diff --git a/core/vm/interpreter.go b/core/vm/interpreter.go
index 9cf0c4e2c1..9fb83799c9 100644
--- a/core/vm/interpreter.go
+++ b/core/vm/interpreter.go
@@ -262,7 +262,7 @@ func (in *EVMInterpreter) Run(contract *Contract, input []byte, readOnly bool) (
                // if the operation clears the return data (e.g. it has returning data)
                // set the last return to the result of the operation.
                if operation.returns {
-                       in.returnData = common.CopyBytes(res)
+                       in.returnData = res
                }
 
                switch {
```

### Statetest to test for the issue

```json
{
  "trigger-issue": {
    "env": {
      "currentCoinbase": "b94f5374fce5edbc8e2a8697c15331677e6ebf0b",
      "currentDifficulty": "0x20000",
      "currentGasLimit": "0x26e1f476fe1e22",
      "currentNumber": "0x1",
      "currentTimestamp": "0x3e8",
      "previousHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
    },
    "pre": {
      "0x00000000000000000000000000000000000000bb": {
        "code": "0x6001600053600260015360036002536004600353600560045360066005536006600260066000600060047f7ef0367e633852132a0ebbf70eb714015dd44bc82e1e55a96ef1389c999c1bcaf13d600060003e596000208055",
        "storage": {},
        "balance": "0x5",
        "nonce": "0x0"
      },
      "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
        "code": "0x",
        "storage": {},
        "balance": "0xffffffff",
        "nonce": "0x0"
      }
    },
    "transaction": {
      "gasPrice": "0x1",
      "nonce": "0x0",
      "to": "0x00000000000000000000000000000000000000bb",
      "data": [
        "0x"
      ],
      "gasLimit": [
        "0x7a1200"
      ],
      "value": [
        "0x01"
      ],
      "secretKey": "0x45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8"
    },
    "out": "0x",
    "post": {
      "Berlin": [
        {
          "hash": "2a38a040bab1e1fa499253d98b2fd363e5756ecc52db47dd59af7116c068368c",
          "logs": "1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
          "indexes": {
            "data": 0,
            "gas": 0,
            "value": 0
          }
        }
      ]
    }
  }
}
```

