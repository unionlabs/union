---
order: 3
---

# Proposer Selection Procedure

This document specifies the Proposer Selection Procedure that is used in Tendermint, the consensus algorithm adopted in CometBFT, to choose a round proposer.
As Tendermint is “leader-based consensus protocol”, the proposer selection is critical for its correct functioning.

At a given block height, the proposer selection algorithm runs with the same validator set at each round .
Between heights, an updated validator set may be specified by the application as part of the ABCIResponses' EndBlock.

## Requirements for Proposer Selection

This sections covers the requirements with Rx being mandatory and Ox optional requirements.
The following requirements must be met by the Proposer Selection procedure:

### R1: Determinism

Given a validator set `V`, and two honest validators `p` and `q`, for each height `h` and each round `r` the following must hold:

  `proposer_p(h,r) = proposer_q(h,r)`

where `proposer_p(h,r)` is the proposer returned by the Proposer Selection Procedure at process `p`, at height `h` and round `r`.

### R2: Fairness

Given a validator set with total voting power P and a sequence S of elections. In any sub-sequence of S with length C*P, a validator v must be elected as proposer P/VP(v) times, i.e. with frequency:

 f(v) ~ VP(v) / P

where C is a tolerance factor for validator set changes with following values:

- C == 1 if there are no validator set changes
- C ~ k when there are validator changes

*[this needs more work]*

## Basic Algorithm

At its core, the proposer selection procedure uses a weighted round-robin algorithm.

A model that gives a good intuition on how/ why the selection algorithm works and it is fair is that of a priority queue. The validators move ahead in this queue according to their voting power (the higher the voting power the faster a validator moves towards the head of the queue). When the algorithm runs the following happens:

- all validators move "ahead" according to their powers: for each validator, increase the priority by the voting power
- first in the queue becomes the proposer: select the validator with highest priority
- move the proposer back in the queue: decrease the proposer's priority by the total voting power

Notation:

- vset - the validator set
- n - the number of validators
- VP(i) - voting power of validator i
- A(i) - accumulated priority for validator i
- P - total voting power of set
- avg - average of all validator priorities
- prop - proposer

Simple view at the Selection Algorithm:

```md
    def ProposerSelection (vset):

        // compute priorities and elect proposer
        for each validator i in vset:
            A(i) += VP(i)
        prop = max(A)
        A(prop) -= P
```

## Stable Set

Consider the validator set:

Validator | p1 | p2
----------|----|---
VP        | 1  | 3

Assuming no validator changes, the following table shows the proposer priority computation over a few runs. Four runs of the selection procedure are shown, starting with the 5th the same values are computed.
Each row shows the priority queue and the process place in it. The proposer is the closest to the head, the rightmost validator. As priorities are updated, the validators move right in the queue. The proposer moves left as its priority is reduced after election.

| Priority   Run | -2 | -1 | 0     | 1  | 2     | 3  | 4  | 5  | Alg step         |
|----------------|----|----|-------|----|-------|----|----|----|------------------|
|                |    |    | p1,p2 |    |       |    |    |    | Initialized to 0 |
| run 1          |    |    |       | p1 |       | p2 |    |    | A(i)+=VP(i)      |
|                |    | p2 |       | p1 |       |    |    |    | A(p2)-= P        |
| run 2          |    |    |       |    | p1,p2 |    |    |    | A(i)+=VP(i)      |
|                | p1 |    |       |    | p2    |    |    |    | A(p1)-= P        |
| run 3          |    | p1 |       |    |       |    |    | p2 | A(i)+=VP(i)      |
|                |    | p1 |       | p2 |       |    |    |    | A(p2)-= P        |
| run 4          |    |    | p1    |    |       |    | p2 |    | A(i)+=VP(i)      |
|                |    |    | p1,p2 |    |       |    |    |    | A(p2)-= P        |

It can be shown that:

- At the end of each run k+1 the sum of the priorities is the same as at end of run k. If a new set's priorities are initialized to 0 then the sum of priorities will be 0 at each run while there are no changes.
- The max distance between priorities is (n-1) *P.*[formal proof not finished]*

## Validator Set Changes

Between proposer selection runs the validator set may change. Some changes have implications on the proposer election.

### Voting Power Change

Consider again the earlier example and assume that the voting power of p1 is changed to 4:

Validator | p1 | p2
----------|----|---
VP        | 4  | 3

Let's also assume that before this change the proposer priorities were as shown in first row (last run). As it can be seen, the selection could run again, without changes, as before.

| Priority   Run | -2 | -1 | 0 | 1  | 2  | Comment           |
|----------------|----|----|---|----|----|-------------------|
| last run       |    | p2 |   | p1 |    | __update VP(p1)__ |
| next run       |    |    |   |    | p2 | A(i)+=VP(i)       |
|                | p1 |    |   |    | p2 | A(p1)-= P         |

However, when a validator changes power from a high to a low value, some other validator remain far back in the queue for a long time. This scenario is considered again in the Proposer Priority Range section.

As before:

- At the end of each run k+1 the sum of the priorities is the same as at run k.
- The max distance between priorities is (n-1) * P.

### Validator Removal

Consider a new example with set:

Validator | p1 | p2 | p3
----------|----|----|---
VP        | 1  | 2  | 3

Let's assume that after the last run the proposer priorities were as shown in first row with their sum being 0. After p2 is removed, at the end of next proposer selection run (penultimate row) the sum of priorities is -2 (minus the priority of the removed process).

The procedure could continue without modifications. However, after a sufficiently large number of modifications in validator set, the priority values would migrate towards maximum or minimum allowed values causing truncations due to overflow detection.
For this reason, the selection procedure adds another __new step__ that centers the current priority values such that the priority sum remains close to 0.

| Priority   Run | -3 | -2 | -1 | 0 | 1  | 2  | 3  | Comment               |
|----------------|----|----|----|---|----|----|----|-----------------------|
| last run       | p3 |    |    |   | p1 | p2 |    | __remove p2__         |
| nextrun        |    |    |    |   |    |    |    |                       |
| __new step__   |    | p3 |    |   |    | p1 |    | A(i) -= avg, avg = -1 |
|                |    |    |    |   | p3 |    | p1 | A(i)+=VP(i)           |
|                |    |    | p1 |   | p3 |    |    | A(p1)-= P             |

The modified selection algorithm is:

```md
    def ProposerSelection (vset):

        // center priorities around zero
        avg = sum(A(i) for i in vset)/len(vset)
        for each validator i in vset:
            A(i) -= avg

        // compute priorities and elect proposer
        for each validator i in vset:
            A(i) += VP(i)
        prop = max(A)
        A(prop) -= P
```

Observations:

- The sum of priorities is now close to 0. Due to integer division the sum is an integer in (-n, n), where n is the number of validators.

### New Validator

When a new validator is added, same problem as the one described for removal appears, the sum of priorities in the new set is not zero. This is fixed with the centering step introduced above.

One other issue that needs to be addressed is the following. A validator V that has just been elected is moved to the end of the queue. If the validator set is large and/ or other validators have significantly higher power, V will have to wait many runs to be elected. If V removes and re-adds itself to the set, it would make a significant (albeit unfair) "jump" ahead in the queue.

In order to prevent this, when a new validator is added, its initial priority is set to:

```md
    A(V) = -1.125 *  P
```

where P is the total voting power of the set including V.

Current implementation uses the penalty factor of 1.125 because it provides a small punishment that is efficient to calculate. See [here](https://github.com/tendermint/tendermint/pull/2785#discussion_r235038971) for more details.

If we consider the validator set where p3 has just been added:

Validator | p1 | p2 | p3
----------|----|----|---
VP        | 1  | 3  | 8

then p3 will start with proposer priority:

```md
    A(p3) = -1.125 * (1 + 3 + 8) ~ -13
```

Note that since current computation uses integer division there is penalty loss when sum of the voting power is less than 8.

In the next run, p3 will still be ahead in the queue, elected as proposer and moved back in the queue.

| Priority   Run | -13 | -9 | -5 | -2 | -1 | 0 | 1 | 2  | 5  | 6  | 7  | Alg step              |
|----------------|-----|----|----|----|----|---|---|----|----|----|----|-----------------------|
| last run       |     |    |    | p2 |    |   |   | p1 |    |    |    | __add p3__            |
|                | p3  |    |    | p2 |    |   |   | p1 |    |    |    | A(p3) = -13           |
| next run       |     | p3 |    |    |    |   |   | p2 |    | p1 |    | A(i) -= avg, avg = -4 |
|                |     |    |    |    | p3 |   |   |    | p2 |    | p1 | A(i)+=VP(i)           |
|                |     |    | p1 |    | p3 |   |   |    | p2 |    |    | A(p1)-=P              |

## Proposer Priority Range

With the introduction of centering, some interesting cases occur. Low power validators that bind early in a set that includes high power validator(s) benefit from subsequent additions to the set. This is because these early validators run through more right shift operations during centering, operations that increase their priority.

As an example, consider the set where p2 is added after p1, with priority -1.125 * 80k = -90k. After the selection procedure runs once:

Validator | p1   | p2   | Comment
----------|------|------|------------------
VP        | 80k  | 10   |
A         | 0    | -90k | __added p2__
A         | 45k  | -45k | __run selection__

Then execute the following steps:

1. Add a new validator p3:

    Validator | p1  | p2 | p3
    ----------|-----|----|---
    VP        | 80k | 10 | 10

2. Run selection once. The notation '..p'/'p..' means very small deviations compared to column priority.

    | Priority  Run | -90k.. | -60k | -45k | -15k | 0 | 45k | 75k | 155k | Comment      |
    |---------------|--------|------|------|------|---|-----|-----|------|--------------|
    | last run      | p3     |      | p2   |      |   | p1  |     |      | __added p3__ |
    | next run
    | *right_shift*|       |  p3  |        |  p2 |   |     |  p1  |        | A(i) -= avg,avg=-30k
    |              |       |  ..p3|        | ..p2|   |     |      |  p1    | A(i)+=VP(i)
    |              |       |  ..p3|        | ..p2|   |     | p1.. |        | A(p1)-=P, P=80k+20

3. Remove p1 and run selection once:

    Validator | p3     | p2    | Comment
    ----------|--------|-------|------------------
    VP        | 10     | 10    |
    A         | -60k   | -15k  |
    A         | -22.5k | 22.5k | __run selection__

At this point, while the total voting power is 20, the distance between priorities is 45k. It will take 4500 runs for p3 to catch up with p2.

In order to prevent these types of scenarios, the selection algorithm performs scaling of priorities such that the difference between min and max values is smaller than two times the total voting power.

The modified selection algorithm is:

```md
    def ProposerSelection (vset):

        // scale the priority values
        diff = max(A)-min(A)
        threshold = 2 * P
     if  diff > threshold:
            scale = diff/threshold
            for each validator i in vset:
          A(i) = A(i)/scale

        // center priorities around zero
        avg = sum(A(i) for i in vset)/len(vset)
        for each validator i in vset:
            A(i) -= avg

        // compute priorities and elect proposer
        for each validator i in vset:
            A(i) += VP(i)
        prop = max(A)
        A(prop) -= P
```

Observations:

- With this modification, the maximum distance between priorities becomes 2 * P.

Note also that even during steady state the priority range may increase beyond 2 * P. The scaling introduced here  helps to keep the range bounded.

## Wrinkles

### Validator Power Overflow Conditions

The validator voting power is a positive number stored as an int64. When a validator is added the `1.125 * P` computation must not overflow. As a consequence the code handling validator updates (add and update) checks for overflow conditions making sure the total voting power is never larger than the largest int64 `MAX`, with the property that `1.125 * MAX` is still in the bounds of int64. Fatal error is return when overflow condition is detected.

### Proposer Priority Overflow/ Underflow Handling

The proposer priority is stored as an int64. The selection algorithm performs additions and subtractions to these values and in the case of overflows and underflows it limits the values to:

```go
    MaxInt64  =  1 << 63 - 1
    MinInt64  = -1 << 63
```

## Requirement Fulfillment Claims

__[R1]__

The proposer algorithm is deterministic giving consistent results across executions with same transactions and validator set modifications.
[WIP - needs more detail]

__[R2]__

Given a set of processes with the total voting power P, during a sequence of elections of length P, the number of times any process is selected as proposer is equal to its voting power. The sequence of the P proposers then repeats. If we consider the validator set:

Validator | p1 | p2
----------|----|---
VP        | 1  | 3

With no other changes to the validator set, the current implementation of proposer selection generates the sequence:
`p2, p1, p2, p2, p2, p1, p2, p2,...` or [`p2, p1, p2, p2`]*
A sequence that starts with any circular permutation of the [`p2, p1, p2, p2`] sub-sequence would also provide the same degree of fairness. In fact these circular permutations show in the sliding window (over the generated sequence) of size equal to the length of the sub-sequence.

Assigning priorities to each validator based on the voting power and updating them at each run ensures the fairness of the proposer selection. In addition, every time a validator is elected as proposer its priority is decreased with the total voting power.

Intuitively, a process v jumps ahead in the queue at most (max(A) - min(A))/VP(v) times until it reaches the head and is elected. The frequency is then:

```md
    f(v) ~ VP(v)/(max(A)-min(A)) = 1/k * VP(v)/P
```

For current implementation, this means v should be proposer at least VP(v) times out of k * P runs, with scaling factor k=2.
