# regalloc2 Design Overview

This document describes the basic architecture of the regalloc2
register allocator. It describes the externally-visible interface
(input CFG, instructions, operands, with their invariants; meaning of
various parts of the output); core data structures; and the allocation
pipeline, or series of algorithms that compute an allocation. It ends
with a description of future work and expectations, as well as an
appendix that notes design influences and similarities to the
IonMonkey backtracking allocator.

# API, Input IR and Invariants

The toplevel API to regalloc2 consists of a single entry point `run()`
that takes a register environment, which specifies all physical
registers, and the input program. The function returns either an error
or an `Output` struct that provides allocations for each operand and a
vector of additional instructions (moves, loads, stores) to insert.

## Register Environment

The allocator takes a `MachineEnv` which specifies, for each of the
two register classes `Int` and `Float`, a vector of `PReg`s by index. A
`PReg` is nothing more than the class and index within the class; the
allocator does not need to know anything more.

The `MachineEnv` provides a vector of preferred and non-preferred
physical registers per class. Any register not in either vector will
not be allocated. Usually, registers that do not need to be saved in
the prologue if used (i.e., caller-save registers) are given in the
"preferred" vector. The environment also provides exactly one scratch
register per class. This register must not be in the preferred or
non-preferred vectors, and is used whenever a set of moves that need
to occur logically in parallel have a cycle (for a simple example,
consider a swap `r0, r1 := r1, r0`).

With some more work, we could potentially remove the need for the
scratch register by requiring support for an additional edit type from
the client ("swap"), but we have not pursued this.

## CFG and Instructions

The allocator operates on an input program that is in a standard CFG
representation: the function body is a sequence of basic blocks, and
each block has a sequence of instructions and zero or more
successors. The allocator also requires the client to provide
predecessors for each block, and these must be consistent with the
successors.

Instructions are opaque to the allocator except for a few important
bits: (1) `is_ret` (is a return instruction); (2) `is_branch` (is a
branch instruction); (3) `is_move` (is a move between registers), and
(4) a vector of Operands, covered below. Every block must end in a
return or branch.

Both instructions and blocks are named by indices in contiguous index
spaces. A block's instructions must be a contiguous range of
instruction indices, and block i's first instruction must come
immediately after block i-1's last instruction.

The CFG must have *no critical edges*. A critical edge is an edge from
block A to block B such that A has more than one successor *and* B has
more than one predecessor. For this definition, the entry block has an
implicit predecessor, and any block that ends in a return has an
implicit successor.

Note that there are *no* requirements related to the ordering of
blocks, and there is no requirement that the control flow be
reducible. Some *heuristics* used by the allocator will perform better
if the code is reducible and ordered in reverse postorder (RPO),
however: in particular, (1) this interacts better with the
contiguous-range-of-instruction-indices live range representation that
we use, and (2) the "approximate loop depth" metric will actually be
exact if both these conditions are met.

## Operands and VRegs

Every instruction operates on values by way of `Operand`s. An operand
consists of the following fields:

- VReg, or virtual register. *Every* operand mentions a virtual
  register, even if it is constrained to a single physical register in
  practice. This is because we track liveranges uniformly by vreg.
  
- Policy, or "constraint". Every reference to a vreg can apply some
  constraint to the vreg at that point in the program. Valid policies are:
  
  - Any location;
  - Any register of the vreg's class;
  - Any stack slot;
  - A particular fixed physical register; or
  - For a def (output), a *reuse* of an input register.
  
- The "kind" of reference to this vreg: Def, Use, Mod. A def
  (definition) writes to the vreg, and disregards any possible earlier
  value. A mod (modify) reads the current value then writes a new
  one. A use simply reads the vreg's value.
  
- The position: before or after the instruction.
  - Note that to have a def (output) register available in a way that
    does not conflict with inputs, the def should be placed at the
    "before" position. Similarly, to have a use (input) register
    available in a way that does not conflict with outputs, the use
    should be placed at the "after" position.

This operand-specification design allows for SSA and non-SSA code (see
section below for details).

VRegs, or virtual registers, are specified by an index and a register
class (Float or Int). The classes are not given separately; they are
encoded on every mention of the vreg. (In a sense, the class is an
extra index bit, or part of the register name.) The input function
trait does require the client to provide the exact vreg count,
however.

Implementation note: both vregs and operands are bit-packed into
u32s. This is essential for memory-efficiency. As a result of the
operand bit-packing in particular (including the policy constraints!),
the allocator supports up to 2^21 (2M) vregs per function, and 2^6
(64) physical registers per class. Later we will also see a limit of
2^20 (1M) instructions per function. These limits are considered
sufficient for the anticipated use-cases (e.g., compiling Wasm, which
also has function-size implementation limits); for larger functions,
it is likely better to use a simpler register allocator in any case.

## Reuses and Two-Address ISAs

Some instruction sets primarily have instructions that name only two
registers for a binary operator, rather than three: both registers are
inputs, and the result is placed in one of the registers, clobbering
its original value. The most well-known modern example is x86. It is
thus imperative that we support this pattern well in the register
allocator.

This instruction-set design is somewhat at odds with an SSA
representation, where a value cannot be redefined. Even in non-SSA
code, it is awkward to overwrite a vreg that may need to be used again
later.

Thus, the allocator supports a useful fiction of sorts: the
instruction can be described as if it has three register mentions --
two inputs and a separate output -- and neither input will be
clobbered. The output, however, is special: its register-placement
policy is "reuse input i" (where i == 0 or 1). The allocator
guarantees that the register assignment for that input and the output
will be the same, so the instruction can use that register as its
"modifies" operand. If the input is needed again later, the allocator
will take care of the necessary copying.

We will see below how the allocator makes this work by doing some
preprocessing so that the core allocation algorithms do not need to
worry about this constraint.

Note that some non-SSA clients, such as Cranelift using the
regalloc.rs-to-regalloc2 compatibility shim, will instead generate
their own copies (copying to the output vreg first) and then use "mod"
operand kinds, which allow the output vreg to be both read and
written. regalloc2 works hard to make this as efficient as the
reused-input scheme by treating moves specially (see below).

## SSA

regalloc2 was originally designed to take an SSA IR as input, where
the usual definitions apply: every vreg is defined exactly once, and
every vreg use is dominated by its one def. (Useing blockparams means
that we do not need additional conditions for phi-nodes.)

The allocator then evolved to support non-SSA inputs as well. As a
result, the input is maximally flexible right now: it does not check
for and enforce, nor try to take advantage of, the single-def
rule. However, blockparams are still available.

In the future, we hope to change this, however, once compilation of
non-SSA inputs is no longer needed. Specifically, if we can migrate
Cranelift to the native regalloc2 API rather than the regalloc.rs
compatibility shim, we will be able to remove "mod" operand kinds,
assume (and verify) single defs, and take advantage of this when
reasoning about various algorithms in the allocator.

## Block Parameters

Every block can have *block parameters*, and a branch to a block with
block parameters must provide values for those parameters via
operands. When a branch has more than one successor, it provides
separate operands for each possible successor. These block parameters
are equivalent to phi-nodes; we chose this representation because they
are in many ways a more consistent representation of SSA. 

To see why we believe block parameters are a slightly nicer design
choice than use of phi nodes, consider: phis are special
pseudoinstructions that must come first in a block, are all defined in
parallel, and whose uses occur on the edge of a particular
predecessor. All of these facts complicate any analysis that scans
instructions and reasons about uses and defs. It is much closer to the
truth to actually put those uses *in* the predecessor, on the branch,
and put all the defs at the top of the block as a separate kind of
def. The tradeoff is that a vreg's def now has two possibilities --
ordinary instruction def or blockparam def -- but this is fairly
reasonable to handle.

## Non-SSA

As mentioned, regalloc2 supports non-SSA inputs as well. No special
flag is needed to place the allocator in this mode or disable SSA
verification. However, we hope to eventually remove this functionality
when it is no longer needed.

## Program Moves

As an especially useful feature for non-SSA IR, regalloc2 supports
special handling of "move" instructions: it will try to merge the
input and output allocations to elide the move altogether.

It turns out that moves are used frequently in the non-SSA input that
we observe from Cranelift via the regalloc.rs compatibility shim. They
are used in three different ways:

- Moves to or from physical registers, used to implement ABI details
  or place values in particular registers required by certain
  instructions.
- Moves between vregs on program edges, as lowered from phi/blockparam
  dataflow in the higher-level SSA IR (CLIF).
- Moves just prior to two-address-form instructions that modify an
  input to form an output: the input is moved to the output vreg to
  avoid clobbering the input.
  
Note that, strictly speaking, special handling of program moves is
redundant because each of these kinds of uses has an equivalent in the
"native" regalloc2 API:

- Moves to/from physical registers can become operand constraints,
  either on a particular instruction that requires/produces the values
  in certain registers (e.g., a call or ret with args/results in regs,
  or a special instruction with fixed register args), or on a ghost
  instruction at the top of function that defs vregs for all in-reg
  args.
  
- Moves between vregs as a lowering of blockparams/phi nodes can be
  replaced with use of regalloc2's native blockparam support.
  
- Moves prior to two-address-form instructions can be replaced with
  the reused-input mechanism.
  
Thus, eventually, special handling of program moves should be
removed. However, it is very important for performance at the moment.

## Output

The allocator produces two main data structures as output: an array of
`Allocation`s and a sequence of edits. Some other data, such as
stackmap slot info, is also provided.

### Allocations

The allocator provides an array of `Allocation` values, one per
`Operand`. Each `Allocation` has a kind and an index. The kind may
indicate that this is a physical register or a stack slot, and the
index gives the respective register or slot. All allocations will
conform to the constraints given, and will faithfully preserve the
dataflow of the input program.

### Inserted Moves

In order to implement the necessary movement of data between
allocations, the allocator needs to insert moves at various program
points.

The vector of inserted moves contains tuples that name a program point
and an "edit". The edit is either a move, from one `Allocation` to
another, or else a kind of metadata used by the checker to know which
VReg is live in a given allocation at any particular time. The latter
sort of edit can be ignored by a backend that is just interested in
generating machine code.

Note that the allocator will never generate a move from one stackslot
directly to another, by design. Instead, if it needs to do so, it will
make use of the scratch register. (Sometimes such a move occurs when
the scratch register is already holding a value, e.g. to resolve a
cycle of moves; in this case, it will allocate another spillslot and
spill the original scratch value around the move.)

Thus, the single "edit" type can become either a register-to-register
move, a load from a stackslot into a register, or a store from a
register into a stackslot.

# Data Structures

We now review the data structures that regalloc2 uses to track its
state.

## Program-Derived Alloc-Invariant Data

There are a number of data structures that are computed in a
deterministic way from the input program and then subsequently used
only as read-only data during the core allocation procedure.

### Livein/Liveout Bitsets

The livein and liveout bitsets (`liveins` and `liveouts` on the `Env`)
are allocated one per basic block and record, per block, which vregs
are live entering and leaving that block. They are computed using a
standard backward iterative dataflow analysis and are exact; they do
not over-approximate (this turns out to be important for performance,
and is also necessary for correctness in the case of stackmaps).

### Blockparam Vectors: Source-Side and Dest-Side

The initialization stage scans the input program and produces two
vectors that represent blockparam flows from branches to destination
blocks: `blockparam_ins` and `blockparam_outs`.

These two vectors are the first instance we will see of a recurring
pattern: the vectors contain tuples that are carefully ordered in a
way such that their sort-order is meaningful. "Build a vector lazily
then sort" is a common idiom: it batches the O(n log n) cost into one
operation that the stdlib has aggressively optimized, it provides
dense storage, and it allows for a scan in a certain order that often
lines up with a scan over the program.

In this particular case, we will build vectors of (vreg, block) points
that are meaningful either at the start or end of a block, so that
later, when we scan over a particular vreg's allocations in block
order, we can generate another vector of allocations. One side (the
"outs") also contains enough information that it can line up with the
other side (the "ins") in a later sort.

To make this work, `blockparam_ins` contains a vector of (to-vreg,
to-block, from-block) tuples, and has an entry for every blockparam of
every block. Note that we can compute this without actually observing
from-blocks; we only need to iterate over `block_preds` at any given
block.

Then, `blockparam_outs` contains a vector of (from-vreg, from-block,
to-block, to-vreg), and has an entry for every parameter on every
branch that ends a block. There is exactly one "out" tuple for every
"in" tuple. As mentioned above, we will later scan over both to
generate moves.

### Program-Move Vectors: Source-Side and Dest-Side

Similar to blockparams, we handle moves specially. In fact, we ingest
all moves in the input program into a set of vectors -- "move sources"
and "move dests", analogous to the "ins" and "outs" blockparam vectors
described above -- and then completely ignore the moves in the program
thereafter. The semantics of the API are such that all program moves
will be recreated with regalloc-inserted edits, and should not still
be emitted after regalloc. This may seem inefficient, but in fact it
allows for better code because it integrates program-moves with the
move resolution that handles other forms of vreg movement. We
previously took the simpler approach of handling program-moves as
opaque instructions with a source and dest, and we found that there
were many redundant move-chains (A->B, B->C) that are eliminated when
everything is handled centrally.

We also construct a `prog_move_merges` vector of live-range index pairs
to attempt to merge when we reach that stage of allocation.

## Core Allocation State: Ranges, Uses, Bundles, VRegs, PRegs

We now come to the core data structures: live-ranges, bundles, virtual
registers and their state, and physical registers and their state.

First we must define a `ProgPoint` precisely: a `ProgPoint` is an
instruction index and a `Before` or `After` suffix. We pack the
before/after suffix into the LSB of a `u32`, so a `ProgPoint` can be
incremented and compared as a simple integer.

A live-range is a contiguous range of program points (half-open,
i.e. including `from` and excluding `to`) for which a particular vreg
is live with a value.

A live-range contains a vector of uses. Each use contains four parts:
the Operand word (directly copied, so there is no need to dereference
it); the ProgPoint at which the use occurs; the operand slot on that
instruction, if any, that the operand comes from, and the use's
'weight". (It's possible to have "ghost uses" that do not derive from
any slot on the instruction.) These four parts are packed into three
`u32`s: the slot can fit in 8 bits, and the weight in 16.

The live-range carries its program-point range, uses, vreg index,
bundle index (see below), and some metadata: spill weight and
flags. The spill weight is the sum of weights of each use. The flags
set currently carries one flag only: whether the live-range starts at
a Def-kind operand. (This is equivalent to whether the range consumes
a value at its start or not.)

Uses are owned only by live-ranges and have no separate identity, but
live-ranges live in a toplevel array and are known by `LiveRangeIndex`
values throughout the allocator. New live-ranges can be created
(e.g. during splitting); old ones are not cleaned up, but rather, all
state is bulk-freed at the end.

Live-ranges are aggregated into "bundles". A bundle is a collection of
ranges that does not overlap. Each bundle carries: a vector (inline
SmallVec) of (range, live-range index) tuples, an allocation (starts
as "none"), a "spillset" (more below), and some metadata, including a
spill weight (sum of ranges' weights), a priority (sum of ranges'
lengths), and three property flags: "minimal", "contains fixed
constraints", "contains stack constraints".

VRegs also contain their vectors of live-ranges, in the same form as a
bundle does (inline SmallVec that has inline (from, to) range bounds
and range indices).

There are two important overlap invariants: (i) no liveranges within a
bundle overlap, and (ii) no liveranges within a vreg overlap. These
are extremely important and we rely on them implicitly in many places.

The live-range vectors in bundles and vregs, and use-vectors in ranges,
have various sorting invariants as well. These invariants differ
according to the phase of the allocator's computation. First, during
live-range construction, live-ranges are placed into vregs in reverse
order (because the computation is a reverse scan) and uses into ranges
in reverse order; these are sorted into forward order at the end of
live-range computation. When bundles are first constructed, their
range vectors are sorted, and they remain so for the rest of allocation,
as we need for interference testing. However, as ranges are created
and split, sortedness of vreg ranges is *not* maintained; they are
sorted once more, in bulk, when allocation is done and we start to
resolve moves.

Finally, we have physical registers. The main data associated with
each is the allocation map. This map is a standard BTree, indexed by
ranges (`from` and `to` ProgPoints) and yielding a LiveRange for each
location range. The ranges have a custom comparison operator defined
that compares equal for any overlap.

This comparison operator allows us to determine whether a range is
free, i.e. has no overlap with a particular range, in one probe -- the
btree will not contain a match. However, it makes iteration over *all*
overlapping ranges somewhat tricky to get right. Notably, Rust's
BTreeMap does not guarantee that the lookup result will be the *first*
equal key, if multiple keys are equal to the probe key. Thus, when we
want to enumerate all overlapping ranges, we probe with a range that
consists of the single program point *before* the start of the actual
query range, using the API that returns an iterator over a range in
the BTree, and then iterate through the resulting iterator to gather
all overlapping ranges (which will be contiguous).

## Spill Bundles

It is worth describing "spill bundles" separately. Every spillset (see
below; a group of bundles that originated from one bundle) optionally
points to a single bundle that we designate the "spill bundle" for
that spillset. Contrary to the name, this bundle is not
unconditionally spilled. Rather, one can see it as a sort of fallback:
it is where liveranges go when we give up on processing them via the
normal backtracking loop, and will only process them once more in the
"second-chance" stage.

This fallback behavior implies that the spill bundle must always be
able to accept a spillslot allocation, i.e., it cannot require a
register. This invariant is what allows spill bundles to be processed
in a different way, after backtracking has completed.

The spill bundle acquires liveranges in two ways. First, as we split
bundles, we will trim the split pieces in certain ways so that some
liveranges are immediately placed in the spill bundle. Intuitively,
the "empty" regions that just carry a value, but do not satisfy any
operands, should be in the spill bundle: it is better to have a single
consistent location for the value than to move it between lots of
different split pieces without using it, as moves carry a cost.

Second, the spill bundle acquires the liveranges of a bundle that has
no requirement to be in a register when that bundle is processed, but
only if the spill bundle already exists. In other words, we won't
create a second-chance spill bundle just for a liverange with an "Any"
use; but if it was already forced into existence by splitting and
trimming, then we might as well use it.

Note that unlike other bundles, a spill bundle's liverange vector
remains unsorted until we do the second-chance allocation. This allows
quick appends of more liveranges.

## Allocation Queue

The allocation queue is simply a priority queue (built with a binary
max-heap) of (prio, bundle-index) tuples.

## Spillsets and Spillslots

Every bundle contains a reference to a spillset. Spillsets are used to
assign spillslots near the end of allocation, but before then, they
are also a convenient place to store information that is common among
*all bundles* that share the spillset. In particular, spillsets are
initially assigned 1-to-1 to bundles after all bundle-merging is
complete; so spillsets represent in some sense the "original bundles",
and as splitting commences, the smaller bundle-pieces continue to
refer to their original spillsets.

We stash some useful information on the spillset because of this: a
register hint, used to create some "stickiness" between pieces of an
original bundle that are assigned separately after splitting; the
spill bundle; the common register class of all vregs in this bundle;
the vregs whose liveranges are contained in this bundle; and then some
information actually used if this is spilled to the stack (`required`
indicates actual stack use; `size` is the spillslot count; `slot` is
the actual stack slot).

Spill *sets* are later allocated to spill *slots*. Multiple spillsets
can be assigned to one spillslot; the only constraint is that
spillsets assigned to a spillslot must not overlap. When we look up
the allocation for a bundle, if the bundle is not given a specific
allocation (its `alloc` field is `Allocation::none()`), this means it
is spilled, and we traverse to the spillset then spillslot.

## Other: Fixups, Stats, Debug Annotations

There are a few fixup vectors that we will cover in more detail
later. Of particular note is the "multi-fixed-reg fixup vector": this
handles instructions that constrain the same input vreg to multiple,
different, fixed registers for different operands at the same program
point. The only way to satisfy such a set of constraints is to
decouple all but one of the inputs (make them no longer refer to the
vreg) and then later insert copies from the first fixed use of the
vreg to the other fixed regs.

The `Env` also carries a statistics structure with counters that are
incremented, which can be useful for evaluating the effects of
changes; and a "debug annotations" hashmap from program point to
arbitrary strings that is filled out with various useful diagnostic
information if enabled, so that an annotated view of the program with
its liveranges, bundle assignments, inserted moves, merge and split
decisions, etc. can be viewed.

# Allocation Pipeline

We now describe the pipeline that computes register allocations.

## Live-range Construction

The first step in performing allocation is to analyze the input
program to understand its dataflow: that is, the ranges during which
virtual registers must be assigned to physical registers. Computing
these ranges is what allows us to do better than a trivial "every vreg
lives in a different location, always" allocation.

We compute precise liveness first using an iterative dataflow
algorithm with BitVecs. (See below for our sparse chunked BitVec
description.) This produces the `liveins` and `liveouts` vectors of
BitVecs per block.

We then perform a single pass over blocks in reverse order, and scan
instructions in each block in reverse order. Why reverse order? We
must see instructions within a block in reverse to properly compute
liveness (a value is live backward from an use to a def). Because we
want to keep liveranges in-order as we build them, to enable
coalescing, we visit blocks in reverse order as well, so overall this
is simply a scan over the whole instruction index space in reverse
order.

For each block, we perform a scan with the following state:

- A liveness bitvec, initialized at the start from `liveouts`.
- A vector of live-range indices, with one entry per vreg, initially
  "invalid" (this vector is allocated once and reused at each block).
- In-progress vector of live-range indices per vreg in the vreg state,
  in *reverse* order (we will reverse it when we're done).

A vreg is live at the current point in the scan if its bit is set in
the bitvec; its entry in the vreg-to-liverange vec may be stale, but
if the bit is not set, we ignore it.

We initially create a liverange for all vregs that are live out of the
block, spanning the whole block. We will trim this below if it is
locally def'd and does not pass through the block.

For each instruction, we process its effects on the scan state:

- For all clobbers (which logically happen at the end of the
  instruction), add a single-program-point liverange to each clobbered
  preg.

- If not a move:
  - for each program point [after, before], for each operand at
    this point(\*):
    - if a def or mod:
      - if not currently live, this is a dead def; create an empty
        LR.
    - if a def:
      - set the start of the LR for this vreg to this point.
      - set as dead.
    - if a use:
      - create LR if not live, with start at beginning of block.
      
- Else, if a move:
  - simple case (no pinned vregs): 
    - add to `prog_move` data structures, and update LRs as above.
    - effective point for the use is *after* the move, and for the mod
      is *before* the *next* instruction. Why not more conventional
      use-before, def-after? Because this allows the move to happen in
      parallel with other moves that the move-resolution inserts
      (between split fragments of a vreg); these moves always happen
      at the gaps between instructions. We place it after, not before,
      because before may land at a block-start and interfere with edge
      moves, while after is always a "normal" gap (a move cannot end a
      block).
  - otherwise: see below (pinned vregs).


(\*) an instruction operand's effective point is adjusted in a few
cases. If the instruction is a branch, its uses (which are
blockparams) are extended to the "after" point. If there is a reused
input, all *other* inputs are extended to "after": this ensures proper
interference (as we explain more below).

We then treat blockparams as defs at the end of the scan (beginning of
the block), and create the "ins" tuples. (The uses for the other side
of the edge are already handled as normal uses on a branch
instruction.)

### Optimization: Pinned VRegs and Moves

In order to efficiently handle the translation from the regalloc.rs
API, which uses named RealRegs that are distinct from VirtualRegs
rather than operand constraints, we need to implement a few
optimizations. The translation layer translates RealRegs as particular
vregs at the regalloc2 layer, because we need to track their liveness
properly. Handling these as "normal" vregs, with massive bundles of
many liveranges throughout the function, turns out to be a very
inefficient solution. So we mark them as "pinned" with a hook in the
RA2 API. Semantically, this means they are always assigned to a
particular preg whenever mentioned in an operand (but *NOT* between
those points; it is possible for a pinned vreg to move all about
registers and stackslots as long as it eventually makes it back to its
home preg in time for its next use).

This has a few implications during liverange construction. First, when
we see an operand that mentions a pinned vreg, we translate this to an
operand constraint that names a fixed preg. Later, when we build
bundles, we will not create a bundle for the pinned vreg; instead we
will transfer its liveranges directly as unmoveable reservations in
pregs' allocation maps. Finally, we need to handle moves specially.

With the caveat that "this is a massive hack and I am very very
sorry", here is how it works. A move between two pinned vregs is easy:
we add that to the inserted-moves vector right away because we know the
Allocation on both sides. A move from a pinned vreg to a normal vreg
is the first interesting case. In this case, we (i) create a ghost def
with a fixed-register policy on the normal vreg, doing the other
liverange-maintenance bits as above, and (ii) adjust the liveranges on
the pinned vreg (so the preg) in a particular way. If the preg is live
flowing downward, then this move implies a copy, because the normal
vreg and the pinned vreg are both used in the future and cannot
overlap. But we cannot keep the preg continuously live, because at
exactly one program point, the normal vreg is pinned to it. So we cut
the downward-flowing liverange just *after* the normal vreg's
fixed-reg ghost def. Then, whether it is live downward or not, we
create an upward-flowing liverange on the pinned vreg that ends just
*before* the ghost def.

The move-from-normal-to-pinned case is similar. First, we create a
ghost use on the normal vreg that pins its value at this program point
to the fixed preg. Then, if the preg is live flowing downward, we trim
its downward liverange to start just after the fixed use.

There are also some tricky metadata-maintenance records that we emit
so that the checker can keep this all straight.

The outcome of this hack, together with the operand-constraint
translation on normal uses/defs/mods on pinned vregs, is that we
essentially are translating regalloc.rs's means of referring to real
registers to regalloc2's preferred abstractions by doing a bit of
reverse-engineering. It is not perfect, but it works. Still, we hope
to rip it all out once we get rid of the need for the compatibility
shim.

### Handling Reused Inputs

Reused inputs are also handled a bit specially. We have already
described how we essentially translate the idiom so that the output's
allocation is used for input and output, and there is a move just
before the instruction that copies the actual input (which will not be
clobbered) to the output. Together with an attempt to merge the
bundles for the two, to elide the move if possible, this works
perfectly well as long as we ignore all of the other inputs.

But we can't do that: we have to ensure that other inputs' allocations
are correct too. Note that using the output's allocation as the input
is actually potentially incorrect if the output is at the After point
and the input is at the Before: the output might share a register with
one of the *other* (normal, non-reused) inputs if that input's vreg
were dead afterward. This will mean that we clobber the other input.

So, to get the interference right, we *extend* all other (non-reused)
inputs of an instruction with a reused input to the After point. This
ensures that the other inputs are *not* clobbered by the slightly
premature use of the output register.

The source has a link to a comment in IonMonkey that implies that it
uses a similar solution to this problem, though it's not entirely
clear.

(This odd dance, like many of the others above and below, is "written
in fuzzbug failures", so to speak. It's not entirely obvious until one
sees the corner case where it's necessary!)

## Bundle Merging

Once we have built the liverange vectors for every vreg, we can reverse
these vectors (recall, they were built in strict reverse order) and
initially assign one bundle per (non-pinned) vreg. We then try to
merge bundles together as long as find pairs of bundles that do not
overlap and that (heuristically) make sense to merge.

Note that this is the only point in the allocation pipeline where
bundles get larger. We initially merge as large as we dare (but not
too large, because then we'll just cause lots of conflicts and
splitting later), and then try out assignments, backtrack via
eviction, and split continuously to chip away at the problem until we
have a working set of allocation assignments.

We attempt to merge three kinds of bundle pairs: reused-input to
corresponding output; across program moves; and across blockparam
assignments.

To merge two bundles, we traverse over both their sorted liverange
vectors at once, checking for overlaps. Note that we can do this without
pointer-chasing to the liverange data; the (from, to) range is in the
liverange vector itself.

We also check whether the merged bundle would have conflicting
requirements (see below for more on requirements). We do a coarse
check first, checking 1-bit flags that indicate whether either bundle
has any fixed-reg constraints or stack-only constraints. If so, we
need to do a detailed check by actually computing merged requirements
on both sides, merging, and checking for Conflict (the lattice bottom
value). If no conflict, we merge.

A performance note: merging is extremely performance-sensitive, and it
turns out that a mergesort-like merge of the liverange vectors is too
expensive, partly because it requires allocating a separate result
vector (in-place merge in mergesort is infamously complex). Instead,
we simply append one vector onto the end of the other and invoke
Rust's builtin sort. We could special-case "one bundle is completely
before the other", but we currently don't do that (performance idea!).

Once all bundles are merged as far as they will go, we compute cached
bundle properties (priorities and weights) and enqueue them on the
priority queue for allocation.

## Recurring: Bundle Property Computation

The core allocation loop is a recurring iteration of the following: we
take the highest-priority bundle from the allocation queue; we compute
its requirements; we try to find it a register according to those
requirements; if no fit, we either evict some other bundle(s) from
their allocations and try again, or we split the bundle and put the
parts back on the queue. We record all the information we need to make
the evict-or-split decision (and where to split) *during* the physical
register allocation-map scans, so we don't need to go back again to
compute that.

Termination is nontrivial to see, because of eviction. How do we
guarantee we don't get into an infinite loop where two bundles fight
over a register forever? In fact, this can easily happen if there is a
bug; we fixed many fuzzbugs like this, and we have a check for
"infinite loop" based on an upper bound on iterations. But if the
allocator is correct, it should never happen.

Termination is guaranteed because (i) bundles always get smaller, (ii)
eviction only occurs when a bundle is *strictly* higher weight (not
higher-or-equal), and (iii) once a bundle gets down to its "minimal"
size, it has an extremely high weight that is guaranteed to evict any
non-minimal bundle. A minimal bundle is one that covers only one
instruction. As long as the input program does not have impossible
constraints that require more than one vreg to exist in one preg, an
allocation problem of all minimal bundles will always have a solution.

## Bundle Processing

Let's now talk about what happens when we take a bundle off the
allocation queue. The three basic outcomes are: allocate; split and
requeue; or evict and try again immediately (and eventually allocate
or split/requeue).

### Properties: Weight, Priority, and Requirements

To process a bundle, we have to compute a few properties. In fact we
will have already computed a few of these beforehand, but we describe
them all here.

- Priority: a bundle's priority determines the order in which it is
  considered for allocation. RA2 defines as the sum of the lengths (in
  instruction index space) of each liverange. This causes the
  allocator to consider larger bundles first, when the allocation maps
  are generally more free; they can always be evicted and split later.

- Weight: a bundle's weight indicates how important (in terms of
  runtime) its uses/register mentions are. In an approximate sense,
  inner loop bodies create higher-weight uses. Fixed register
  constraints add some weight, and defs add some weight. Finally,
  weight is divided by priority, so a very large bundle that happens
  to have a few important uses does not unformly exert its weight
  across its entire range. This has the effect of causing bundles to
  be more important (more likely to evict others) the more they are
  split.
  
- Requirement: a bundle's requirement is a value in a lattice that we
  have defined, where top is "Unknown" and bottom is
  "Conflict". Between these two, we have: any register (of a class);
  any stackslot (of a class); a particular register. "Any register"
  can degrade to "a particular register", but any other pair of
  different requirements meets to Conflict. Requirements are derived
  from the operand constraints for all uses in all liveranges in a
  bundle, and then merged with the lattice meet-function.
  
The lattice is as follows (diagram simplified to remove multiple
classes and multiple fixed registers which parameterize nodes; any two
differently-parameterized values are unordered with respect to each
other):

```plain

        ___Unknown_____
        |      |      |
        |      |      |
        | ____Any(rc) |
        |/     |      |
   Stack(rc)  FixedReg(reg)
         \    /
        Conflict
```

Once we have the Requirement for a bundle, we can decide what to do.

### No-Register-Required Cases

If the requirement indicates that no register is needed (`Unknown` or
`Any`, i.e. a register or stack slot would be OK), *and* if the spill
bundle already exists for this bundle's spillset, then we move all the
liveranges over to the spill bundle, as described above.

If the requirement indicates that the stack is needed explicitly
(e.g., for a safepoint), we set our spillset as "required" (this will
cause it to allocate a spillslot) and return; because the bundle has
no other allocation set, it will look to the spillset's spillslot by
default.

If the requirement indicates a conflict, we immediately split and
requeue the split pieces. This split is performed at the point at
which the conflict is first introduced, i.e. just before the first use
whose requirement, when merged into the requirement for all prior uses
combined, goes to `Conflict`. In this way, we always guarantee forward
progress. Note also that a bundle can reach this stage with a
conflicting requirement only if the original liverange had conflicting
uses (e.g., a liverange from a def in a register to a use on stack, or
a liverange between two different fixed-reg-constrained operands); our
bundle merging logic explicitly avoids merging two bundles if it would
create a conflict.

### Allocation-Map Probing

If we did not immediately dispose of the bundle as described above,
then we *can* use a register (either `Any`, which accepts a register
as one of several options, or `Reg`, which must have one, or `Fixed`,
which must have a particular one).

We determine which physical registers whose allocation maps we will
probe, and in what order. If a particular fixed register is required,
we probe only that register. Otherwise, we probe all registers in the
required class.

The order in which we probe, if we are not constrained to a single
register, is carefully chosen. First, if there is a hint register from
the spillset (this is set by the last allocation into a register of
any other bundle in this spillset), we probe that. Then, we probe all
preferred registers; then all non-preferred registers.

For each of the preferred and non-preferred register sequences, we
probe in an *offset* manner: we start at some index partway through
the sequence, determined by some heuristic number that is random and
well-distributed. (In practice, we use the sum of the bundle index and
the instruction index of the start of the first range in the bundle.)
We then march through the sequence and wrap around, stopping before we
hit our starting point again.

The purpose of this offset is to distribute the contention and speed
up the allocation process. In the common case where there are enough
registers to hold values without spilling (for small functions), we
are more likely to choose a free register right away if we throw the
dart at random than if we start *every* probe at register 0, in
order. This has a large allocation performance impact in practice.

For each register in probe order, we probe the allocation map, and
gather, simultaneously, several results: (i) whether the entire range
is free; (ii) if not, the vector of all conflicting bundles, *and* the
highest weight among those bundles; (iii) if not, the *first* conflict
point.

We do this by iterating over all liveranges in the preg's btree that
overlap with each range in the current bundle. This iteration is
somewhat subtle due to multiple "equal" keys (see above where we
describe the use of the btree). It is also adaptive for performance
reasons: it initially obtains an iterator into the btree corresponding
to the start of the first range in the bundle, and concurrently
iterates through both the btree and the bundle. However, if there is a
large gap in the bundle, this might require skipping many irrelevant
entries in the btree. So, if we skip too many entries (heuristically,
16, right now), we do another lookup from scratch in the btree for the
start of the next range in the bundle. This balances between the two
cases: dense bundle, where O(1) iteration through the btree is faster,
and sparse bundle, where O(log n) lookup for each entry is better.

### Decision: Allocate, Evict, or Split

First, the "allocate" case is easy: if, during our register probe
loop, we find a physical register whose allocations do not overlap
this bundle, then we allocate this register; done!

If not, then we need to decide whether to evict some conflicting
bundles and retry, or to split the current bundle into smaller pieces
that may have better luck.

A bit about our split strategy first: contrary to the IonMonkey
allocator which inspired much of our design, we do *not* have a list
of split strategies that split one bundle into many pieces at
once. Instead, each iteration of the allocation loop splits at most
*once*. This simplifies the splitting code greatly, but also turns out
to be a nice heuristic: we split at the point that the bundle first
encounters a conflict for a particular preg assignment, then we hint
that preg for the first (pre-conflict) piece when we retry. In this
way, we always make forward progress -- one piece of the bundle is
always allocated -- and splits are informed by the actual situation at
hand, rather than best guesses. Also note that while this may appear
at first to be a greedy algorithm, it still allows backtracking: the
first half of the split bundle, which we *can* now assign to a preg,
does not necessarily remain on that preg forever (it can still be
evicted later). It is just a split that is known to make at least one
part of the allocation problem solvable.

To determine whether to split or evict, we track our best options: as
we probe, we track the "lowest cost eviction option", which is a set
of bundles and the maximum weight in that set of bundles. We also
track the "lowest cost split option", which is the cost (more below),
the point at which to split, and the register for this option.

For each register we probe, if there is a conflict but none of the
conflicts are fixed allocations, we receive a vector of bundles that
conflicted, and also separately, the first conflicting program
point. We update the lowest-cost eviction option if the cost (max
weight) of the conflicting bundles is less than the current best. We
update the lowest-cost split option if the cost is less as well,
according to the following definition of cost: a split's cost is the
cost of its move, as defined by the weight of a normal def operand at
the split program point, plus the cost of all bundles beyond the split
point (which will still be conflicts even after the split).

If there is a conflict with a fixed allocation, then eviction is not
an option, but we can still compute the candidate split point and cost
in the same way as above.

Finally, as an optimization, we pass in the current best cost to the
btree probe inner loop; if, while probing, we have already exceeded
the best cost, we stop early (this improves allocation time without
affecting the result).

Once we have the best cost for split and evict options, we split if
(i) the bundle is not already a minimal bundle, and (ii) we've already
evicted once in this toplevel iteration without success, or the weight
of the current bundle is less than the eviction cost. We then requeue
*both* resulting halves of the bundle with the preg that resulted in
this option as the register hint. Otherwise, we evict all conflicting
bundles and try again.

Note that the split cost does not actually play into the above (split
vs. evict) decision; it is only used to choose *which* split is
best. This is equivalent to saying: we never evict if the current
bundle is less important than the evicted bundles, even if the split
is more expensive still. This is important for forward progress, and
the case where the split would be even more expensive should be very
very rare (it would have to come from a costly move in the middle of
an inner loop).

### How to Split

The actual split procedure is fairly simple. We are given a bundle and
a split-point. We create a new bundle to take on the second half
("rest") of the original. We find the point in the liverange vector
that corresponds to the split, and distribute appropriately. If the
split-point lands in the middle of a liverange, then we split that
liverange as well.

In the case that a new liverange is created, we add the liverange to
the corresponding vreg liverange vector as well. Note that, as described
above, the vreg's liverange vector is unsorted while splitting is
occurring (because we do not need to traverse it or do any lookups
during this phase); so we just append.

The splitting code also supports a "minimal split", in which it simply
peels off the first use. This is used to ensure forward progress when
a bundle has conflicting requirements within it (see above).

#### Spill Bundle and Splitting

Once a split occurs, however, it turns out that we can improve results
by doing a little cleanup. Once we distribute a bundle's liveranges
across two half-bundles, we postprocess by trimming a bit.

In particular, if we see that the "loose ends" around the split point
extend beyond uses, we will create and move ranges to a spill
bundle. That is: if the last liverange in the first-half bundle
extends beyond its last use, we trim that part off into an empty (no
uses) liverange and place that liverange in the spill
bundle. Likewise, if the first liverange in the second-half bundle
starts before its first use, we trim that part off into an empty
liverange and place it in the spill bundle.

This is, empirically, an improvement: it reduces register contention
and makes splitting more effective. The intuition is twofold: (i) it
is better to put all of the "flow-through" parts of a vreg's liveness
into one bundle that is never split, and can be spilled to the stack
if needed, to avoid unnecessary moves; and (ii) if contention is high
enough to cause splitting, it is more likely there will be an actual
stack spill, and if this is the case, it is better to do the store
just after the last use and reload just before the first use of the
respective bundles.

Unfortunately, this heuristic choice does interact somewhat poorly
with program moves: moves between two normal (non-pinned) vregs do not
create ghost uses or defs, and so these points of the ranges can be
spilled, turning a normal register move into a move from or to the
stack. However, empirically, we have found that adding such ghost
uses/defs actually regresses some cases as well, because it pulls
values back into registers when we could have had a stack-to-stack
move (that might even be a no-op if the same spillset); overall, it
seems better to trim. It also improves allocation performance by
reducing contention in the registers during the core loop (before
second-chance allocation).

## Second-Chance Allocation: Spilled Bundles

Once the main allocation loop terminates, when all bundles have either
been allocated or punted to the "spilled bundles" vector, we do
second-chance allocation. This is a simpler loop that never evicts and
never splits. Instead, each bundle gets one second chance, in which it
can probe pregs and attempt to allocate. If it fails, it will actually
live on the stack.

This is correct because we are careful to only place bundles on the
spilled-bundles vector that are *allowed* to live on the
stack. Specifically, only the canonical spill bundles (which will
contain only empty ranges) and other bundles that have an "any" or
"unknown" requirement are placed here (but *not* "stack" requirements;
those *must* be on the stack, so do not undergo second-chance
allocation).

At the end of this process, we have marked spillsets as required
whenever at least one bundle in the spillset actually requires a stack
slot. We can then allocate slots to the spillsets.

## Spillslot Allocation

We must allocate space on the stack, denoted by an abstract index
space, to each spillset that requires it, and for the liveranges in
which it requires it.

To facilitate this, we keep a btree per spillslot in the same way we
do per preg. We will allocate spillsets to slots in a way that avoids
interference.

Note that we actually overapproximate the required ranges for each
spillset in order to improve the behavior of a later phase (redundant
move elimination). Specifically, when we allocate a slot for a
spillset, we reserve that slot for *all* of the liveranges of *every*
vreg that is assigned to that spillset (due to merging rules that
initially merge one-vreg bundles into final merged bundles, there will
be no overlaps here). In other words, we rule out interleaving of
completely different values in the same slot, though bundle merging
does mean that potentially many (non-interfering) vregs may share
it. This provides the important property that if a vreg has been
reloaded, but not modified, its spillslot *still contains the
up-to-date value* (because the slot is reserved for all liveranges of
the vreg). This enables us to avoid another store to the spillslot
later if there is another spilled range.

We perform probing in a way that is somewhat different than for
registers, because the spillslot space is conceptually infinite. We
can thus optimize for slightly better allocation performance by giving
up and allocating a new slot at any time.

For each size class, we keep a linked list of slots. When we need to
allocate a spillset to a slot, we traverse down the list and try a
fixed number of slots. If we find one that fits the spillset's ranges,
we allocate, and we remove the slot from its current place in the list
and append to the end. In this way, it is deprioritized from probing
"for a while", which tends to reduce contention. This is a simple way
to round-robin between slots. If we don't find one that fits after a
fixed number of probes, we allocate a new slot.

And with that, we have valid allocations for all vregs for all points
that they are live! Now we just need to modify the program to reify
these choices.

## Allocation Assignment

The first step in reifying the allocation is to iterate through all
mentions of a vreg and fill in the resulting `Allocation` array with
the appropriate allocations. We do this by simply traversing
liveranges per vreg, looking up the allocation by observing the bundle
(and spillset if no specific allocation for the bundle), and for each
use, filling in the slot according to the saved progpoint/slot info in
the use data.

## Move Generation

The more difficult half of the reification step is generating the
*moves* that will put the values in the right spots.

There are two sources of moves that we must generate. The first are
moves between different ranges of the same vreg, as the split pieces
of that vreg's original bundle may have been assigned to different
locations. The second are moves that result from move semantics in the
input program: either assignments from blockparam args on branches to
the target block's params, or program move instructions. (Recall that
we reify program moves in a unified way with all other moves, so the
client should not generate any machine code for their original moves
in the pre-allocation program.)

Moves are tricky to handle efficiently because they join two
potentially very different locations in the program (in the case of
control-flow-edge moves). In order to avoid the need for random
lookups, which are a cache-locality nightmare even if we have O(log n)
lookups, we instead take a scan-sort-scan approach.

First, we scan over each vreg's liveranges, find the allocation for
each, and for each move that comes *to* or *from* this liverange,
generate a "half-move". The key idea is that we generate a record for
each "side" of the move, and these records are keyed in a way that
after a sort, the "from" and "to" ends will be consecutive. We can
sort the vector of halfmoves once (this is expensive, but not as
expensive as many separate pointer-chasing lookups), then scan it
again to actually generate the move instructions.

To enable the sort to work, half-moves are sorted by a key that is
equivalent to the tuple (from-block, to-block, to-vreg, kind), where
`kind` is "source" or "dest". For each key, the payload is an
allocation. The fields in this tuple are carefully chosen: we know all
of them at every location we generate a halfmove, without expensive
lookups, and sorting by this key will make the source and all dests
(there can be more than one) contiguous in the final order.

Half-moves are generated for several situations. First, at the start
of every block covered by a liverange, we can generate "dest"
half-moves for blockparams, and at the end of every block covered by a
liverange, we can generate "source" half-moves for blockparam args on
branches. Incidentally, this is the reason that `blockparam_ins` and
`blockparam_outs` are sorted tuple-vectors whose tuples begin with
(vreg, block, ...): this is the order in which we do the toplevel scan
over allocations.

Second, at every block edge, if the vreg is live in any pred (at
block-start) or succ (at block-end), we generate a half-move to
transfer the vreg to its own location in the connected block.

This completes the "edge-moves". We sort the half-move array and then
have all of the alloc-to-alloc pairs on a given (from-block, to-block)
edge.

There are also two kinds of moves that happen within blocks. First,
when a live-range ends and another begins for the same vreg in the
same block (i.e., a split in the middle of a block), we know both
sides of the move immediately (because it is the same vreg and we can
look up the adjacent allocation easily), and we can generate that
move.

Second, program moves occur within blocks. Here we need to do a
similar thing as for block-edge half-moves, but keyed on program point
instead. This is why the `prog_move_srcs` and `prog_move_dsts` arrays
are initially sorted by their (vreg, inst) keys: we can directly fill
in their allocation slots during our main scan. Note that when sorted
this way, the source and dest for a given move instruction will be at
different indices. After the main scan, we *re-sort* the arrays by
just the instruction, so the two sides of a move line up at the same
index; we can then traverse both arrays, zipped together, and generate
moves.

Finally, we generate moves to fix up multi-fixed-reg-constraint
situations, and make reused inputs work, as described earlier.

## Move Resolution

During this whole discussion, we have described "generating moves",
but we have not said what that meant. Note that in many cases, there
are several moves at a particular program point that semantically
happen *in parallel*. For example, if multiple vregs change
allocations between two instructions, all of those moves happen as
part of one parallel permutation. Similarly, blockparams have
parallel-assignment semantics. We thus enqueue all the moves that we
generate at program points and resolve them into sequences of
sequential moves that can actually be lowered to move instructions in
the machine code.

First, a word on *move priorities*. There are different kinds of moves
that are generated between instructions, and we have to ensure that
some happen before others, i.e., *not* in parallel. For example, a
vreg might change allocation (due to a split) before an instruction,
then be copied to an output register for an output with a reused-input
policy. The latter move must happen *after* the vreg has been moved
into its location for this instruction.

To enable this, we define "move priorities", which are a logical
extension of program points (i.e., they are sub-points) that enable
finer-grained ordering of moves. We currently have the following
priorities:

- In-edge moves, to place edge-moves before the first instruction in a
  block.
- Block-param metadata, used for the checker only.
- Regular, used for vreg movement between allocations.
- Post-regular, used for checker metadata related to pinned-vreg moves.
- Multi-fixed-reg, used for moves that handle the
  single-vreg-in-multiple-fixed-pregs constraint case.
- Reused-input, used for implementing outputs with reused-input policies.
- Out-edge moves, to place edge-moves after the last instruction
  (prior to the branch) in a block.

Every move is statically given one of these priorities by the code
that generates it.

We collect moves with (prog-point, prio) keys, and we sort by those
keys. We then have, for each such key, a set of moves that
semantically happen in parallel.

We then resolve those moves using a parallel-move resolver, as we now
describe.

### Parallel-Move Resolver

The fundamental issue that arises when resolving parallel moves to
sequential moves is *overlap*: some of the moves may overwrite
registers that other moves use as sources. We must carefully order
moves so that this does not clobber values incorrectly.

We first check if such overlap occurs. If it does not (this is
actually the most common case), the sequence of parallel moves can be
emitted as sequential moves directly. Done!

Otherwise, we have to order the moves carefully. Furthermore, if there
is a *cycle* anywhere among the moves, we will need a scratch
register. (Consider, e.g., t0 := t1 and t1 := t0 in parallel: with
only move instructions and no direct "exchange" instruction, we cannot
reify this without a third register.)

We first compute a mapping from each move instruction to the move
instruction, if any, that it must precede. Note that there can be only
one such move for a given move, because each destination can be
written only once; so a move might be constrained only before the one
move that overwrites its source. (This will be important in a bit!)

Our task is now to find an ordering of moves that respects these
dependencies. To do so, we perform a depth-first search on the graph
induced by the dependencies, which will generate a sequence of
sequential moves in reverse order. We keep a stack of moves; we start
with any move that has not been visited yet; in each iteration, if the
top-of-stack has no out-edge to another move (does not need to come
before any others), then push it to a result vector, followed by all
others on the stack (in popped order). If it does have an out-edge and
the target is already visited and not on the stack anymore (so already
emitted), likewise, emit this move and the rest on the stack. If it
has an out-edge to a move not yet visited, push on the stack and
continue. Otherwise, if out-edge to a move currently on the stack, we
have found a cycle. In this case, we emit the moves on the stack with
a modification: the first move writes to a scratch register, and we
emit an additional move that moves from the scratch to the first
move's dest. This breaks the cycle.

The astute reader may notice that this sounds like a canonical
application of Tarjan's algorithm for finding SCCs (strongly-connected
components). Why don't we have the full complexity of that algorithm?
In particular, *why* can we emit the cycle *right away* once we find
it, rather than ensuring that we have gotten all of the SCC first?

The answer is that because there is only *one* out-edge at most (a
move can only require preceding *one* other move), all SCCs must be
simple cycles. This means that once we have found a cycle, no other
nodes (moves) can be part of the SCC, because every node's single
out-edge is already accounted for. This is what allows us to avoid a
fully general SCC algorithm.

Once the vector of moves in-reverse has been constructed, we reverse
it and return.

Note that this "move resolver" is fuzzed separately with a simple
symbolic move simulator (the `moves` fuzz-target).

### Stack-to-Stack Moves

There is one potentially difficult situation that could arise from the
move-resolution logic so far: if a vreg moves from one spillslot to
another, this implies a memory-to-memory move, which most machine
architectures cannot handle natively. It would be much nicer if we
could ensure within the regalloc that this never occurs.

This is in fact possible to do in a postprocessing step. We iterate
through the sequential moves, tracking whether the scratch register is
in use (has been written). When we see a stack-to-stack move: (i) if
the scratch register is not in use, generate a stack-to-scratch move
and scratch-to-stack move; otherwise, (ii) if the scratch register is
in use, allocate an "extra spillslot" if one has not already been
allocated, move the scratch reg to that, do the above stack-to-scratch
/ scratch-to-stack sequence, then reload the scratch reg from the
extra spillslot.

## Redundant-Spill/Load Elimination

As a final step before returning the vector of program edits to the
client, we perform one optimization: redundant-spill/load elimination.

To understand the need for this, consider what will occur when a vreg
is (i) defined once, (ii) used many times, and (iii) spilled multiple
times between some of the uses: with the design described above, we
will move the value from the preg to the stack after every segment of
uses, and then reload it when the next use occurs. However, only the
first spill is actually needed; as we noted above, we allocate
spillslots so that the slot that corresponded to the vreg at the first
spill will always be reserved for that vreg as long as it is live. If
no other defs or mods occur, the value in the slot can be reloaded,
and need not be written back every time.

This inefficiency is a result of our invariant that a vreg lives in
exactly one place at a time, and these locations are joined by
moves. This is a simple and effective design to use for most of the
allocation pipeline, but falls flat here. It is especially inefficient
when the unnecessary spill occurs in an inner loop. (E.g.: value
defined at top of function is spilled, then used once in the middle of
an inner loop body.)

The opposite case can also sometimes occur, though it is rarer: a
value is loaded into a register, spilled, and then reloaded into the
same register. This can happen when hinting is successful at getting
several segments of a vreg to use the same preg, but splitting has
trimmed part of the liverange between uses and put it in the spill
bundle, and the spill bundle did not get a reg.

In order to resolve this inefficiency, we implement a general
redundant-spill/load elimination pass (an even more general solution
would be a full redundant-move elimination pass, but we focus on moves
that are spills/loads to contain the complexity for now). This pass
tracks, for every allocation (reg or spillslot), whether it is a copy
of another allocation. This state is invalidated whenever either that
allocation or the allocation of which it is a copy is
overwritten. When we see a move instruction, if the destination is
already a copy of the source, we elide the move. (There are some
additional complexities to preserve checker metadata which we do not
describe here.)

Note that this could, in principle, be done as a fixpoint analysis
over the CFG; it must be, if we try to preserve state across
blocks. This is because a location is only a copy of another if that
is true on every incoming edge. However, to avoid the cost and
complexity of doing such an analysis, we instead take the much simpler
approach of doing only an intra-block analysis. This turns out to be
sufficient to remove most redundant moves, especially in the common
case of a single use of an otherwise-spilled value.

Note that we could do better *if* we accepted only SSA code, because
we would know that a value could not be redefined once written. We
should consider this again once we clean up and remove the non-SSA
support.

# Future Plans

## SSA-Only Cleanup

When the major user (Cranelift via the regalloc.rs shim) migrates to
generate SSA code and native regalloc2 operands, there are many bits
of complexity we can remove, as noted throughout this
writeup. Briefly, we could (i) remove special handling of program
moves, (ii) remove the pinned-vreg hack, (iii) simplify redundant-move
elimination, (iv) remove special handling of "mod" operands, and (v)
probably simplify plenty of code given the invariant that a def always
starts a range.

More importantly, we expect this change to result in potentially much
better allocation performance. The use of special pinned vregs and
moves to/from them instead of fixed-reg constraints, explicit moves
for every reused-input constraint, and already-sequentialized series
of move instructions on edges for phi nodes, are all expensive ways of
encoding regalloc2's native input primitives that have to be
reverse-engineered. Removing that translation layer would be
ideal. Also, allowing regalloc2 to handle phi-node (blockparam)
lowering in a way that is integrated with other moves will likely
generate better code than the way that program-move handling interacts
with Cranelift's manually lowered phi-moves at the moment.

## Better Split Heuristics

We have spent quite some effort trying to improve splitting behavior,
and it is now generally decent, but more work could be done here,
especially with regard to the interaction between splits and the loop
nest.

## Native Debuginfo Output

Cranelift currently computes value locations (in registers and
stack-slots) for detailed debuginfo with an expensive post-pass, after
regalloc is complete. This is because the existing register allocator
does not support returning this information directly. However,
providing such information by generating it while we scan over
liveranges in each vreg would be relatively simple, and has the
potential to be much faster and more reliable for Cranelift. We should
investigate adding an interface for this to regalloc2 and using it.

# Appendix: Comparison to IonMonkey Allocator

There are a number of differences between the [IonMonkey
allocator](https://searchfox.org/mozilla-central/source/js/src/jit/BacktrackingAllocator.cpp)
and this one. While this allocator initially began as an attempt to
clone IonMonkey's, it has drifted significantly as we optimized the
design (especially after we built the regalloc.rs shim and had to
adapt to its code style); it is easier at this point to name the
similarities than the differences.

* The core abstractions of "liverange", "bundle", "vreg", "preg", and
  "operand" (with policies/constraints) are the same.
  
* The overall allocator pipeline is the same, and the top-level
  structure of each stage should look similar. Both allocators begin
  by computing liveranges, then merging bundles, then handling bundles
  and splitting/evicting as necessary, then doing second-chance
  allocation, then reifying the decisions.
  
* The cost functions are very similar, though the heuristics that make
  decisions based on them are not.

Several notable high-level differences are:

* There are [fuzz/fuzz_targets/](many different fuzz targets) that
  exercise the allocator, including a full symbolic checker
  (`ion_checker` target) based on the [symbolic checker in
  regalloc.rs](https://cfallin.org/blog/2021/03/15/cranelift-isel-3/)
  and, e.g., a targetted fuzzer for the parallel move-resolution
  algorithm (`moves`) and the SSA generator used for generating cases
  for the other fuzz targets (`ssagen`).

* The data-structure invariants are simplified. While the IonMonkey
  allocator allowed for LiveRanges and Bundles to overlap in certain
  cases, this allocator sticks to a strict invariant: ranges do not
  overlap in bundles, and bundles do not overlap. There are other
  examples too: e.g., the definition of minimal bundles is very simple
  and does not depend on scanning the code at all. In general, we
  should be able to state simple invariants and see by inspection (as
  well as fuzzing -- see above) that they hold.
  
* The data structures themselves are simplified. Where IonMonkey uses
  linked lists in many places, this allocator stores simple inline
  smallvecs of liveranges on bundles and vregs, and smallvecs of uses
  on liveranges. We also (i) find a way to construct liveranges
  in-order immediately, without any need for splicing, unlike
  IonMonkey, and (ii) relax sorting invariants where possible to allow
  for cheap append operations in many cases.
  
* The splitting heuristics are significantly reworked. Whereas
  IonMonkey has an all-at-once approach to splitting an entire bundle,
  and has a list of complex heuristics to choose where to split, this
  allocator does conflict-based splitting, and tries to decide whether
  to split or evict and which split to take based on cost heuristics.
  
* The liverange computation is exact, whereas IonMonkey approximates
  using a single-pass algorithm that makes vregs live across entire
  loop bodies. We have found that precise liveness improves allocation
  performance and generated code quality, even though the liveness
  itself is slightly more expensive to compute.
  
* Many of the algorithms in the IonMonkey allocator are built with
  helper functions that do linear scans. These "small quadratic" loops
  are likely not a huge issue in practice, but nevertheless have the
  potential to be in corner cases. As much as possible, all work in
  this allocator is done in linear scans.
  
* There are novel schemes for solving certain interesting design
  challenges. One example: in IonMonkey, liveranges are connected
  across blocks by, when reaching one end of a control-flow edge in a
  scan, doing a lookup of the allocation at the other end. This is in
  principle a linear lookup (so quadratic overall). We instead
  generate a vector of "half-moves", keyed on the edge and from/to
  vregs, with each holding one of the allocations. By sorting and then
  scanning this vector, we can generate all edge moves in one linear
  scan. There are a number of other examples of simplifications: for
  example, we handle multiple conflicting
  physical-register-constrained uses of a vreg in a single instruction
  by recording a copy to do in a side-table, then removing constraints
  for the core regalloc. Ion instead has to tweak its definition of
  minimal bundles and create two liveranges that overlap (!) to
  represent the two uses.
  
* Using block parameters rather than phi-nodes significantly
  simplifies handling of inter-block data movement. IonMonkey had to
  special-case phis in many ways because they are actually quite
  weird: their uses happen semantically in other blocks, and their
  defs happen in parallel at the top of the block. Block parameters
  naturally and explicitly reprsent these semantics in a direct way.

* The allocator supports irreducible control flow and arbitrary block
  ordering (its only CFG requirement is that critical edges are
  split).
  
* The allocator supports non-SSA code, and has native support for
  handling program moves specially.

# Appendix: Performance-Tuning Lessons

In the course of optimizing the allocator's performance, we found a
number of general principles:

* We got substantial performance speedups from using vectors rather
  than linked lists everywhere. This is well-known, but nevertheless,
  it took some thought to work out how to avoid the need for any
  splicing, and it turns out that even when our design is slightly
  less efficient asymptotically (e.g., apend-and-re-sort rather than
  linear-time merge of two sorted liverange lists when merging
  bundles), it is faster.

* We initially used a direct translation of IonMonkey's splay tree as
  an allocation map for each PReg. This turned out to be significantly
  (!) less efficient than Rust's built-in BTree data structures, for
  the usual cache-efficiency vs. pointer-chasing reasons.
  
* We initially used dense bitvecs, as IonMonkey does, for
  livein/liveout bits. It turned out that a chunked sparse design (see
  below) was much more efficient.

* Precise liveness significantly improves performance because it
  reduces the size of liveranges (i.e., interference), and probing
  registers with liveranges is the most significant hot inner
  loop. Paying a fraction of a percent runtime for the iterative
  dataflow algorithm to get precise bitsets is more than worth it.

* The randomized probing of registers was a huge win: as above, the
  probing is very expensive, and reducing the average number of probes
  it takes to find a free register is very important.

* In general, single-pass algorithms and design of data structures to
  enable them are important. For example, the half-move technique
  avoids the need to do any O(log n) search at all, and is relatively
  cache-efficient. As another example, a side-effect of the precise
  liveness was that we could then process operands within blocks in
  actual instruction order (in reverse), which allowed us to simply
  append liveranges to in-progress vreg liverange vectors and then
  reverse at the end. The expensive part is a single pass; only the
  bitset computation is a fixpoint loop.
  
* Sorts are better than always-sorted data structures (like btrees):
  they amortize all the comparison and update cost to one phase, and
  this phase is much more cache-friendly than a bunch of spread-out
  updates.

* Take care of basic data structures and their operator definitions!
  We initially used the auto-derived comparator on ProgPoint, and let
  ProgPoint be a normal struct (with a u32 inst index and a
  Befor/After enum). The comparator for this, used in many sorting
  inner loops, was a compound thing with conditionals. Instead, pack
  them in a u32 and do a simple compare (and save half the memory as
  well). Likewise, the half-move key is a single value packed in a
  u64; this is far more efficient than the tuple comparator on a
  4-tuple, and the half-move sort (which can be a few percent or more
  of total allocation time) became multiple times cheaper.

# Appendix: Data Structure: Chunked Sparse BitVec

We use a "chunked sparse bitvec" to store liveness information, which
is just a set of VReg indices. The design is fairly simple: the
toplevel is a HashMap from "chunk" to a `u64`, and each `u64`
represents 64 contiguous indices.

The intuition is that while the vreg sets are likely sparse overall,
they will probably be dense within small regions of the index
space. For example, in the Nth block in a function, the values that
flow from block N-1 will largely be almost-contiguous vreg indices, if
vregs are allocated in sequence down the function body. Or, at least,
they will be some local vregs together with a few defined at the top
of the function; two separate chunks will cover that.

We tried a number of other designs as well. Initially we used a simple
dense bitvec, but this was prohibitively expensive: O(n^2) space when
the real need is closer to O(n) (i.e., a classic sparse matrix). We
also tried a hybrid scheme that kept a vector of indices when small
and used either a bitvec or a hashset when large. This did not perform
as well because (i) it was less memory-efficient (the chunking helps
with this) and (ii) insertions are more expensive when they always
require a full hashset/hashmap insert.

# Appendix: Fuzzing

We have five fuzz targets: `ssagen`, `domtree`, `moves`, `ion`, and
`ion_checker`.

## SSAGen

The SSAGen target tests our SSA generator, which generates cases for
the full allocator fuzz targets. The SSA generator is careful to
always generate a valid CFG, with split critical edges, and valid SSA,
so that we never have to throw out a test input before we reach the
allocator itself. (An alternative fuzzing approach randomly generates
programs and then throws out those that do not meet certain conditions
before using them as legitimate testcases; this is much simpler, but
less efficient.)

To generate a valid CFG, with no unreachable blocks and with no
critical edges, the generator (i) glues together units of either one
or three blocks (A->B, A->C), forming either a straight-through
section or a conditional. These are glued together into a "spine", and
the conditionals (the "C" block), where they exist, are then linked to
a random target block chosen among the main blocks of these one- or
three-block units. The targets are chosen either randomly, for
potentially irreducible CFGs, or in a way that ensures proper nesting
of loop backedges, if a structured CFG is requested.

SSA is generated by first choosing which vregs will be defined in each
block, and which will be defined as blockparams vs. instruction
defs. Instructions are then generated, with operands chosen among the
"available" vregs: those defined so far in the current block and all
of those in any other block that dominates this one.

The SSAGen fuzz target runs the above code generator against an SSA
validator, and thus ensures that it will only generate valid SSA code.

## Domtree

The `domtree` fuzz target computes dominance using the algorithm that
we use elsewhere in our CFG analysis, and then walks a
randomly-generated path through the CFG. It checks that the dominance
definition ("a dom b if any path from entry to b must pass through a")
is consistent with this particular randomly-chosen path.

## Moves

The `moves` fuzz target tests the parallel move resolver. It generates
a random sequence of parallel moves, careful to ensure that each
destination is written only once. It then runs the parallel move
resolver, and then *abstractly interprets* the resulting sequential
series of moves, thus determining which inputs flow to which
outputs. This must match the original set of parallel moves.

## Ion and Ion-checker

The `ion` fuzz target runs the allocator over test programs generated
by SSAGen. It does not validate the output; it only tests that the
allocator runs to completion and does not panic. This was used mainly
during development, and is now less useful than the checker-based
target.

The `ion_checker` fuzz target runs the allocator's result through a
symbolic checker, which is adapted from the one developed for
regalloc.rs (see [this blog
post](https://cfallin.org/blog/2021/01/22/cranelift-isel-2/) for more
details). This is the most useful fuzz target in the fuzzing suite,
and has found many bugs in development.
