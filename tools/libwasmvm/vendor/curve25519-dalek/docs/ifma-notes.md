An AVX512-IFMA implementation of the vectorized point operation
strategy.

# IFMA instructions

AVX512-IFMA is an extension to AVX-512 consisting of two instructions:

* `vpmadd52luq`: packed multiply of unsigned 52-bit integers and add
  the low 52 product bits to 64-bit accumulators;
* `vpmadd52huq`: packed multiply of unsigned 52-bit integers and add
  the high 52 product bits to 64-bit accumulators;

These operate on 64-bit lanes of their source vectors, taking the low
52 bits of each lane of each source vector, computing the 104-bit
products of each pair, and then adding either the high or low 52 bits
of the 104-bit products to the 64-bit lanes of the destination vector.
The multiplication is performed internally by reusing circuitry for
floating-point arithmetic.  Although these instructions are part of
AVX512, the AVX512VL (vector length) extension (present whenever IFMA
is) allows using them with 512, 256, or 128-bit operands.

This provides a major advantage to vectorized integer operations:
previously, vector operations could only use a \\(32 \times 32
\rightarrow 64\\)-bit multiplier, while serial code could use a
\\(64\times 64 \rightarrow 128\\)-bit multiplier.

## IFMA for big-integer multiplications

A detailed example of the intended use of the IFMA instructions can be
found in a 2016 paper by Gueron and Krasnov, [_Accelerating Big
Integer Arithmetic Using Intel IFMA Extensions_][2016_gueron_krasnov].
The basic idea is that multiplication of large integers (such as 1024,
2048, or more bits) can be performed as follows.

First, convert a “packed” 64-bit representation
\\[
\begin{aligned}
x &= x'_0 + x'_1 2^{64} + x'_2 2^{128} + \cdots \\\\
y &= y'_0 + y'_1 2^{64} + y'_2 2^{128} + \cdots 
\end{aligned}
\\]
into a “redundant” 52-bit representation
\\[
\begin{aligned}
x &= x_0 + x_1 2^{52} + x_2 2^{104} + \cdots \\\\
y &= y_0 + y_1 2^{52} + y_2 2^{104} + \cdots 
\end{aligned}
\\]
with each \\(x_i, y_j\\) in a 64-bit lane.

Writing the product as \\(z = z_0 + z_1 2^{52} + z_2 2^{104} + \cdots\\),
the “schoolbook” multiplication strategy gives
\\[
\begin{aligned}
&z_0   &&=& x_0      &   y_0    &           &          &           &          &           &          &        &       \\\\
&z_1   &&=& x_1      &   y_0    &+ x_0      &   y_1    &           &          &           &          &        &       \\\\
&z_2   &&=& x_2      &   y_0    &+ x_1      &   y_1    &+ x_0      &   y_2    &           &          &        &       \\\\
&z_3   &&=& x_3      &   y_0    &+ x_2      &   y_1    &+ x_1      &   y_2    &+ x_0      &   y_3    &        &       \\\\
&z_4   &&=& \vdots\\;&\\;\vdots &+ x_3      &   y_1    &+ x_2      &   y_2    &+ x_1      &   y_3    &+ \cdots&       \\\\
&z_5   &&=&          &          &  \vdots\\;&\\;\vdots &+ x_3      &   y_2    &+ x_2      &   y_3    &+ \cdots&       \\\\
&z_6   &&=&          &          &           &          &  \vdots\\;&\\;\vdots &+ x_3      &   y_3    &+ \cdots&       \\\\
&z_7   &&=&          &          &           &          &           &          &  \vdots\\;&\\;\vdots &+ \cdots&       \\\\
&\vdots&&=&          &          &           &          &           &          &           &          &  \ddots&       \\\\
\end{aligned}
\\]
Notice that the product coefficient \\(z_k\\), representing the value
\\(z_k 2^{52k}\\), is the sum of all product terms
\\(
(x_i 2^{52 i}) (y_j 2^{52 j})
\\)
with \\(k = i + j\\).  
Write the IFMA operators \\(\mathrm{lo}(a,b)\\), denoting the low
\\(52\\) bits of \\(ab\\), and
\\(\mathrm{hi}(a,b)\\), denoting the high \\(52\\) bits of 
\\(ab\\).  
Now we can rewrite the product terms as
\\[
\begin{aligned}
(x_i 2^{52 i}) (y_j 2^{52 j})
&=
2^{52 (i+j)}(
\mathrm{lo}(x_i, y_j) +
\mathrm{hi}(x_i, y_j) 2^{52}
)
\\\\
&=
\mathrm{lo}(x_i, y_j) 2^{52 (i+j)} + 
\mathrm{hi}(x_i, y_j) 2^{52 (i+j+1)}.
\end{aligned}
\\]
This means that the low half of \\(x_i y_j\\) can be accumulated onto
the product limb \\(z_{i+j}\\) and the high half can be directly
accumulated onto the next-higher product limb \\(z_{i+j+1}\\) with no
additional operations.  This allows rewriting the schoolbook
multiplication into the form
\\[
\begin{aligned}
&z_0   &&=& \mathrm{lo}(x_0,&y_0)      &                 &          &                 &          &                 &          &                 &          &        &     \\\\
&z_1   &&=& \mathrm{lo}(x_1,&y_0)      &+\mathrm{hi}(x_0,&y_0)      &+\mathrm{lo}(x_0,&y_1)      &                 &          &                 &          &        &     \\\\
&z_2   &&=& \mathrm{lo}(x_2,&y_0)      &+\mathrm{hi}(x_1,&y_0)      &+\mathrm{lo}(x_1,&y_1)      &+\mathrm{hi}(x_0,&y_1)      &+\mathrm{lo}(x_0,&y_2)      &        &     \\\\
&z_3   &&=& \mathrm{lo}(x_3,&y_0)      &+\mathrm{hi}(x_2,&y_0)      &+\mathrm{lo}(x_2,&y_1)      &+\mathrm{hi}(x_1,&y_1)      &+\mathrm{lo}(x_1,&y_2)      &+ \cdots&     \\\\
&z_4   &&=&        \vdots\\;&\\;\vdots &+\mathrm{hi}(x_3,&y_0)      &+\mathrm{lo}(x_3,&y_1)      &+\mathrm{hi}(x_2,&y_1)      &+\mathrm{lo}(x_2,&y_2)      &+ \cdots&     \\\\
&z_5   &&=&                 &          &        \vdots\\;&\\;\vdots &        \vdots\\;&\\;\vdots &+\mathrm{hi}(x_3,&y_1)      &+\mathrm{lo}(x_3,&y_2)      &+ \cdots&     \\\\
&z_6   &&=&                 &          &                 &          &                 &          &        \vdots\\;&\\;\vdots &        \vdots\\;&\\;\vdots &+ \cdots&     \\\\
&\vdots&&=&                 &          &                 &          &                 &          &                 &          &                 &          &  \ddots&     \\\\
\end{aligned}
\\]
Gueron and Krasnov implement multiplication by constructing vectors
out of the columns of this diagram, so that the source operands for
the IFMA instructions are of the form \\((x_0, x_1, x_2, \ldots)\\) 
and \\((y_i, y_i, y_i, \ldots)\\). 
After performing the multiplication,
the product terms \\(z_i\\) are then repacked into a 64-bit representation.

## An alternative strategy

The strategy described above is aimed at big-integer multiplications,
such as 1024, 2048, or 4096 bits, which would be used for applications
like RSA.  However, elliptic curve cryptography uses much smaller field
sizes, such as 256 or 384 bits, so a different strategy is needed.

The parallel Edwards formulas provide parallelism at the level of the
formulas for curve operations.  This means that instead of scanning
through the terms of the source operands and parallelizing *within* a
field element (as described above), we can arrange the computation in
product-scanning form and parallelize *across* field elements (as
described below).

The parallel Edwards
formulas provide 4-way parallelism, so they can be implemented using
256-bit vectors using a single 64-bit lane for each element, or using
512-bit vectors using two 64-bit lanes.
The only available CPU supporting IFMA (the
i3-8121U) executes 512-bit IFMA instructions at half rate compared to
256-bit instructions, so for now there's no throughput advantage to
using 512-bit IFMA instructions, and this implementation uses 256-bit
vectors.

To extend this to 512-bit vectors, it's only only necessary to achieve
2-way parallelism, and it's possible (with a small amount of overhead)
to create a hybrid strategy that operates entirely within 128-bit
lanes.  This means that cross-lane operations can use the faster
`vpshufd` (1c latency) instead of a general shuffle instruction (3c
latency).

# Choice of radix

The inputs to IFMA instructions are 52 bits wide, so the radix \\(r\\)
used to represent a multiprecision integer must be \\( r \leq 52 \\).
The obvious choice is the "native" radix \\(r = 52\\).

As described above, this choice
has the advantage that for \\(x_i, y_j \in [0,2^{52})\\), the product term
\\[
\begin{aligned}
(x_i 2^{52 i}) (y_j 2^{52 j})
&=
2^{52 (i+j)}(
\mathrm{lo}(x_i, y_j) +
\mathrm{hi}(x_i, y_j) 2^{52}
)
\\\\
&=
\mathrm{lo}(x_i, y_j) 2^{52 (i+j)} + 
\mathrm{hi}(x_i, y_j) 2^{52 (i+j+1)},
\end{aligned}
\\]
so that the low and high halves of the product can be directly accumulated 
onto the product limbs.
In contrast, when using a smaller radix \\(r = 52 - k\\), 
the product term has the form
\\[
\begin{aligned}
(x_i 2^{r i}) (y_j 2^{r j})
&=
2^{r (i+j)}(
\mathrm{lo}(x_i, y_j) +
\mathrm{hi}(x_i, y_j) 2^{52}
)
\\\\
&=
\mathrm{lo}(x_i, y_j) 2^{r (i+j)} + 
(
\mathrm{hi}(x_i, y_j) 2^k
)
2^{r (i+j+1)}.
\end{aligned}
\\]
What's happening is that the product \\(x_i y_j\\) of size \\(2r\\)
bits is split not at \\(r\\) but at \\(52\\), so \\(k\\) product bits
are placed into the low half instead of the high half.  This means
that the high half of the product cannot be directly accumulated onto
\\(z_{i+j+1}\\), but must first be multiplied by \\(2^k\\) (i.e., left
shifted by \\(k\\)).  In addition, the low half of the product is
\\(52\\) bits large instead of \\(r\\) bits.

## Handling offset product terms

[Drucker and Gueron][2018_drucker_gueron] analyze the choice of radix
in the context of big-integer squaring, outlining three ways to handle
the offset product terms, before concluding that all of them are
suboptimal:

1. Shift the results after accumulation;
2. Shift the input operands before multiplication;
3. Split the MAC operation, accumulating into a zeroed register,
   shifting the result, and then adding.
   
The first option is rejected because it could double-shift some
previously accumulated terms, the second doesn't work because the
inputs could become larger than \\(52\\) bits, and the third requires
additional instructions to handle the shifting and adding.

Based on an analysis of total number of instructions, they suggest an
addition to the instruction set, which they call `FMSA` (fused
multiply-shift-add). This would shift the result according to an 8-bit
immediate value before accumulating it into the destination register.

However, this change to the instruction set doesn't seem to be
necessary.  Instead, the product terms can be grouped according to
their coefficients, accumulated together, then shifted once before
adding them to the final sum.  This uses an extra register, shift, and
add, but only once per product term (accumulation target), not once
per source term (as in the Drucker-Gueron paper).

Moreover, because IFMA instructions execute only on two ports
(presumably 0 and 1), while adds and shifts can execute on three ports
(0, 1, and 5), the adds and shifts can execute independently of the
IFMA operations, as long as there is not too much pressure on port 5.
This means that, although the total number of instructions increases,
the shifts and adds do not necessarily increase the execution time, as
long as throughput is limited by IFMA operations.

Finally, because IFMA instructions have 4 cycle latency and 0.5/1
cycle throughput (for 256/512 bit vectors), maximizing IFMA throughput
requires either 8 (for 256) or 4 (for 512) independent operations.  So
accumulating groups of terms independently before adding them at the
end may be necessary anyways, in order to prevent long chains of
dependent instructions.

## Advantages of a smaller radix

Using a smaller radix has other advantages.  Although radix \\(52\\)
is an unsaturated representation from the point of view of the
\\(64\\)-bit accumulators (because up to 4096 product terms can be
accumulated without carries), it's a saturated representation from the
point of view of the multiplier (since \\(52\\)-bit values are the
maximum input size).

Because the inputs to a multiplication must have all of their limbs
bounded by \\(2^{52}\\), limbs in excess of \\(2^{52}\\) must be
reduced before they can be used as an input.  The
[Gueron-Krasnov][2016_gueron_krasnov] paper suggests normalizing
values using a standard, sequential carry chain: for each limb, add
the carryin from reducing the previous limb, compute the carryout and
reduce the current limb, then move to the next limb.

However, when using a smaller radix, such as \\(51\\), each limb can
store a carry bit and still be used as the input to a multiplication.
This means that the inputs do not need to be normalized, and instead
of using a sequential carry chain, we can compute all carryouts in
parallel, reduce all limbs in parallel, and then add the carryins in
parallel (possibly growing the limb values by one bit).

Because the output of this partial reduction is an acceptable
multiplication input, we can "close the loop" using partial reductions
and never have to normalize to a canonical representation through the
entire computation, in contrast to the Gueron-Krasnov approach, which
converts back to a packed representation after every operation.  (This
idea seems to trace back to at least as early as [this 1999
paper][1999_walter]).

Using \\(r = 51\\) is enough to keep a carry bit in each limb and
avoid normalizations.  What about an even smaller radix?  One reason
to choose a smaller radix would be to align the limb boundaries with
an inline reduction (for instance, choosing \\(r = 43\\) for the
Mersenne field \\(p = 2^{127} - 1\\)), but for \\(p = 2^{255 - 19}\\),
\\(r = 51 = 255/5\\) is the natural choice.

# Multiplication

The inputs to a multiplication are two field elements
\\[
\begin{aligned}
x &= x_0 + x_1 2^{51} + x_2 2^{102} + x_3 2^{153} + x_4 2^{204} \\\\
y &= y_0 + y_1 2^{51} + y_2 2^{102} + y_3 2^{153} + y_4 2^{204},
\end{aligned}
\\]
with limbs in range \\([0,2^{52})\\).  

Writing the product terms as
\\[
\begin{aligned}
z &= z_0 + z_1 2^{51} + z_2 2^{102} + z_3 2^{153} + z_4 2^{204} \\\\
  &+ z_5 2^{255} + z_6 2^{306} + z_7 2^{357} + z_8 2^{408} + z_9 2^{459},
\end{aligned}
\\]
a schoolbook multiplication in product scanning form takes the form
\\[
\begin{aligned}
z_0 &= x_0 y_0 \\\\
z_1 &= x_1 y_0 + x_0 y_1 \\\\
z_2 &= x_2 y_0 + x_1 y_1 + x_0 y_2 \\\\
z_3 &= x_3 y_0 + x_2 y_1 + x_1 y_2 + x_0 y_3 \\\\
z_4 &= x_4 y_0 + x_3 y_1 + x_2 y_2 + x_1 y_3 + x_0 y_4 \\\\
z_5 &=           x_4 y_1 + x_3 y_2 + x_2 y_3 + x_1 y_4 \\\\
z_6 &=                     x_4 y_2 + x_3 y_3 + x_2 y_4 \\\\
z_7 &=                               x_4 y_3 + x_3 y_4 \\\\
z_8 &=                                         x_4 y_4 \\\\
z_9 &= 0 \\\\
\end{aligned}
\\]
Each term \\(x_i y_j\\) can be written in terms of IFMA operations as
\\[
x_i y_j = \mathrm{lo}(x_i,y_j) + 2\mathrm{hi}(x_i,y_j)2^{51}.
\\]
Substituting this equation into the schoolbook multiplication, then
moving terms to eliminate the \\(2^{51}\\) factors gives
\\[
\begin{aligned}
z_0 &= \mathrm{lo}(x_0, y_0) \\\\
 &+ \qquad 0 \\\\
z_1 &= \mathrm{lo}(x_1, y_0) + \mathrm{lo}(x_0, y_1) \\\\
 &+ \qquad 2( \mathrm{hi}(x_0, y_0) )\\\\
z_2 &= \mathrm{lo}(x_2, y_0) + \mathrm{lo}(x_1, y_1) + \mathrm{lo}(x_0, y_2) \\\\
 &+ \qquad 2( \mathrm{hi}(x_1, y_0) + \mathrm{hi}(x_0, y_1) )\\\\
z_3 &= \mathrm{lo}(x_3, y_0) + \mathrm{lo}(x_2, y_1) + \mathrm{lo}(x_1, y_2) + \mathrm{lo}(x_0, y_3) \\\\
 &+ \qquad 2( \mathrm{hi}(x_2, y_0) + \mathrm{hi}(x_1, y_1) + \mathrm{hi}(x_0, y_2) )\\\\
z_4 &= \mathrm{lo}(x_4, y_0) + \mathrm{lo}(x_3, y_1) + \mathrm{lo}(x_2, y_2) + \mathrm{lo}(x_1, y_3) + \mathrm{lo}(x_0, y_4) \\\\
 &+ \qquad 2( \mathrm{hi}(x_3, y_0) + \mathrm{hi}(x_2, y_1) + \mathrm{hi}(x_1, y_2) + \mathrm{hi}(x_0, y_3) )\\\\
z_5 &=                         \mathrm{lo}(x_4, y_1) + \mathrm{lo}(x_3, y_2) + \mathrm{lo}(x_2, y_3) + \mathrm{lo}(x_1, y_4) \\\\
 &+ \qquad 2( \mathrm{hi}(x_4, y_0) + \mathrm{hi}(x_3, y_1) + \mathrm{hi}(x_2, y_2) + \mathrm{hi}(x_1, y_3) + \mathrm{hi}(x_0, y_4) )\\\\
z_6 &=                                                 \mathrm{lo}(x_4, y_2) + \mathrm{lo}(x_3, y_3) + \mathrm{lo}(x_2, y_4) \\\\
 &+ \qquad 2(                         \mathrm{hi}(x_4, y_1) + \mathrm{hi}(x_3, y_2) + \mathrm{hi}(x_2, y_3) + \mathrm{hi}(x_1, y_4) )\\\\
z_7 &=                                                                         \mathrm{lo}(x_4, y_3) + \mathrm{lo}(x_3, y_4) \\\\
 &+ \qquad 2(                                                 \mathrm{hi}(x_4, y_2) + \mathrm{hi}(x_3, y_3) + \mathrm{hi}(x_2, y_4) )\\\\
z_8 &=                                                                                                 \mathrm{lo}(x_4, y_4) \\\\
 &+ \qquad 2(                                                                         \mathrm{hi}(x_4, y_3) + \mathrm{hi}(x_3, y_4) )\\\\
z_9 &= 0 \\\\
 &+ \qquad 2(                                                                                                 \mathrm{hi}(x_4, y_4) )\\\\
\end{aligned}
\\]
As noted above, our strategy will be to multiply and accumulate the
terms with coefficient \\(2\\) separately from those with coefficient
\\(1\\), before combining them at the end.  This can alternately be
thought of as accumulating product terms into a *doubly-redundant*
representation, with two limbs for each digit, before collapsing 
the doubly-redundant representation by shifts and adds.

This computation requires 25 `vpmadd52luq` and 25 `vpmadd52huq`
operations.  For 256-bit vectors, IFMA operations execute on an
i3-8121U with latency 4 cycles, throughput 0.5 cycles, so executing 50
instructions requires 25 cycles' worth of throughput.  Accumulating
terms with coefficient \\(1\\) and \\(2\\) seperately means that the
longest dependency chain has length 5, so the critical path has length
20 cycles and the bottleneck is throughput.

# Reduction modulo \\(p\\)

The next question is how to handle the reduction modulo \\(p\\).
Because \\(p = 2^{255} - 19\\), \\(2^{255} = 19 \pmod p\\), so we can
alternately write
\\[
\begin{aligned}
z &= z_0 + z_1 2^{51} + z_2 2^{102} + z_3 2^{153} + z_4 2^{204} \\\\
  &+ z_5 2^{255} + z_6 2^{306} + z_7 2^{357} + z_8 2^{408} + z_9 2^{459}
\end{aligned}
\\]
as
\\[
\begin{aligned}
z &= (z_0 + 19z_5) + (z_1 + 19z_6) 2^{51} + (z_2 + 19z_7) 2^{102} + (z_3 + 19z_8) 2^{153} + (z_4 + 19z_9) 2^{204}.
\end{aligned}
\\]
When using a \\(64 \times 64 \rightarrow 128\\)-bit multiplier, this
can be handled (as in [Ed25519][ed25519_paper]) by premultiplying
source terms by \\(19\\).  Since \\(\lg(19) < 4.25\\), this increases
their size by less than \\(4.25\\) bits, and the rest of the
multiplication can be shown to work out.

Here, we have at most \\(1\\) bit of headroom.  In order to allow
premultiplication, we would need to use radix \\(2^{47}\\), which
would require six limbs instead of five.  Instead, we compute the high
terms \\(z_5, \ldots, z_9\\), each using two chains of IFMA
operations, then multiply by \\(19\\) and combine with the lower terms
\\(z_0, \ldots, z_4\\).  There are two ways to perform the
multiplication by \\(19\\): using more IFMA operations, or using the
`vpmullq` instruction, which computes the low \\(64\\) bits of a \\(64
\times 64\\)-bit product.  However, `vpmullq` has 15c/1.5c
latency/throughput, in contrast to the 4c/0.5c latency/throughput of
IFMA operations, so it seems like a worse choice.

The high terms \\(z_5, \ldots, z_9\\) are sums of \\(52\\)-bit terms,
so they are larger than \\(52\\) bits.  Write these terms in radix \\(52\\) as
\\[
z_{5+i} = z_{5+i}' + z_{5+i}'' 2^{52}, \qquad z_{5+i}' < 2^{52}.
\\]
Then the contribution of \\(z_{5+i}\\), taken modulo \\(p\\), is
\\[
\begin{aligned}
z_{5+i} 2^{255} 2^{51 i}
&= 
19 (z_{5+i}' + z_{5+i}'' 2^{52}) 2^{51 i} 
\\\\
&= 
19 z_{5+i}' 2^{51 i} + 2 \cdot 19 z_{5+i}'' 2^{51 (i+1)}
\\\\
\end{aligned}
\\]
The products \\(19 z_{5+i}', 19 z_{5+i}''\\) can be written in terms of IFMA operations as
\\[
\begin{aligned}
19 z_{5+i}' &= \mathrm{lo}(19, z_{5+i}') + 2 \mathrm{hi}(19, z_{5+i}') 2^{51}, \\\\
19 z_{5+i}'' &= \mathrm{lo}(19, z_{5+i}'') + 2 \mathrm{hi}(19, z_{5+i}'') 2^{51}. \\\\
\end{aligned}
\\]
Because \\(z_{5+i} < 2^{64}\\), \\(z_{5+i}'' < 2^{12} \\), so \\(19
z_{5+i}'' < 2^{17} < 2^{52} \\) and \\(\mathrm{hi}(19, z_{5+i}'') = 0\\).
Because IFMA operations ignore the high bits of their source
operands, we do not need to compute \\(z\_{5+i}'\\) explicitly:
the high bits will be ignored.
Combining these observations, we can write
\\[
\begin{aligned}
z_{5+i} 2^{255} 2^{51 i}
&= 
19 z_{5+i}' 2^{51 i} + 2 \cdot 19 z_{5+i}'' 2^{51 (i+1)}
\\\\
&= 
\mathrm{lo}(19, z_{5+i}) 2^{51 i}
\+ 2 \mathrm{hi}(19, z_{5+i}) 2^{51 (i+1)}
\+ 2 \mathrm{lo}(19, z_{5+i}/2^{52}) 2^{51 (i+1)}.
\end{aligned}
\\]

For \\(i = 0,1,2,3\\), this allows reducing \\(z_{5+i}\\) onto
\\(z_{i}, z_{i+1}\\), and if the low terms are computed using a
doubly-redundant representation, no additional shifts are needed to
handle the \\(2\\) coefficients.  For \\(i = 4\\), there's a
complication: the contribution becomes
\\[
\begin{aligned}
z_{9} 2^{255} 2^{204}
&= 
\mathrm{lo}(19, z_{9}) 2^{204}
\+ 2 \mathrm{hi}(19, z_{9}) 2^{255}
\+ 2 \mathrm{lo}(19, z_{9}/2^{52}) 2^{255}
\\\\
&= 
\mathrm{lo}(19, z_{9}) 2^{204}
\+ 2 \mathrm{hi}(19, z_{9}) 19
\+ 2 \mathrm{lo}(19, z_{9}/2^{52}) 19
\\\\
&=
\mathrm{lo}(19, z_{9}) 2^{204}
\+ 2 
\mathrm{lo}(19, \mathrm{hi}(19, z_{9}) + \mathrm{lo}(19, z_{9}/2^{52})).
\\\\
\end{aligned}
\\]

It would be possible to cut the number of multiplications from 3 to 2
by carrying the high part of each \\(z_i\\) onto \\(z_{i+1}\\). This
would eliminate 5 multiplications, clearing 2.5 cycles of port
pressure, at the cost of 5 additions, adding 1.66 cycles of port
pressure.  But doing this would create a dependency between terms
(e.g., \\(z_{5}\\) must be computed before the reduction of
\\(z_{6}\\) can begin), whereas with the approach above, all
contributions to all terms are computed independently, to maximize ILP
and flexibility for the processor to schedule instructions.

This strategy performs 16 IFMA operations, adding two IFMA operations
to each of the \\(2\\)-coefficient terms and one to each of the
\\(1\\)-coefficient terms.  Considering the multiplication and
reduction together, we use 66 IFMA operations, requiring 33 cycles'
throughput, while the longest chain of IFMA operations is in the
reduction of \\(z_5\\) onto \\(z_1\\), of length 7 (so 28 cycles, plus
2 cycles to combine the two parts of \\(z_5\\), and the bottleneck is
again throughput.

Once this is done, we have computed the product terms
\\[
z = z_0 + z_1 2^{51} + z_2 2^{102} + z_3 2^{153} + z_4 2^{204},
\\]
without reducing the \\(z_i\\) to fit in \\(52\\) bits.  Because the
overall flow of operations alternates multiplications and additions or
subtractions, we would have to perform a reduction after an addition
but before the next multiplication anyways, so there's no benefit to
fully reducing the limbs at the end of a multiplication.  Instead, we
leave them unreduced, and track the reduction state using the type
system to ensure that unreduced limbs are not accidentally used as an
input to a multiplication.

# Squaring

Squaring operates similarly to multiplication, but with the
possibility to combine identical terms.
As before, we write the input as
\\[
\begin{aligned}
x &= x_0 + x_1 2^{51} + x_2 2^{102} + x_3 2^{153} + x_4 2^{204}
\end{aligned}
\\]
with limbs in range \\([0,2^{52})\\).
Writing the product terms as
\\[
\begin{aligned}
z &= z_0 + z_1 2^{51} + z_2 2^{102} + z_3 2^{153} + z_4 2^{204} \\\\
  &+ z_5 2^{255} + z_6 2^{306} + z_7 2^{357} + z_8 2^{408} + z_9 2^{459},
\end{aligned}
\\]
a schoolbook squaring in product scanning form takes the form
\\[
\begin{aligned}
z_0 &=   x_0 x_0 \\\\
z_1 &= 2 x_1 x_0 \\\\
z_2 &= 2 x_2 x_0 +   x_1 x_1 \\\\
z_3 &= 2 x_3 x_0 + 2 x_2 x_1 \\\\
z_4 &= 2 x_4 x_0 + 2 x_3 x_1 + x_2 x_2 \\\\
z_5 &= 2 x_4 x_1 + 2 x_3 x_2 \\\\
z_6 &= 2 x_4 x_2 +   x_3 x_3 \\\\
z_7 &= 2 x_4 x_3 \\\\
z_8 &=   x_4 x_4 \\\\
z_9 &= 0 \\\\
\end{aligned}
\\]
As before, we write \\(x_i x_j\\) as
\\[
x_i x_j = \mathrm{lo}(x_i,x_j) + 2\mathrm{hi}(x_i,x_j)2^{51},
\\]
and substitute to obtain
\\[
\begin{aligned}
z_0 &=   \mathrm{lo}(x_0, x_0) + 0 \\\\
z_1 &= 2 \mathrm{lo}(x_1, x_0) + 2 \mathrm{hi}(x_0, x_0) \\\\
z_2 &= 2 \mathrm{lo}(x_2, x_0) +   \mathrm{lo}(x_1, x_1) + 4 \mathrm{hi}(x_1, x_0) \\\\
z_3 &= 2 \mathrm{lo}(x_3, x_0) + 2 \mathrm{lo}(x_2, x_1) + 4 \mathrm{hi}(x_2, x_0) + 2 \mathrm{hi}(x_1, x_1) \\\\
z_4 &= 2 \mathrm{lo}(x_4, x_0) + 2 \mathrm{lo}(x_3, x_1) +   \mathrm{lo}(x_2, x_2) + 4 \mathrm{hi}(x_3, x_0) + 4 \mathrm{hi}(x_2, x_1) \\\\
z_5 &= 2 \mathrm{lo}(x_4, x_1) + 2 \mathrm{lo}(x_3, x_2) + 4 \mathrm{hi}(x_4, x_0) + 4 \mathrm{hi}(x_3, x_1) + 2 \mathrm{hi}(x_2, x_2) \\\\
z_6 &= 2 \mathrm{lo}(x_4, x_2) +   \mathrm{lo}(x_3, x_3) + 4 \mathrm{hi}(x_4, x_1) + 4 \mathrm{hi}(x_3, x_2) \\\\
z_7 &= 2 \mathrm{lo}(x_4, x_3) + 4 \mathrm{hi}(x_4, x_2) + 2 \mathrm{hi}(x_3, x_3) \\\\
z_8 &=   \mathrm{lo}(x_4, x_4) + 4 \mathrm{hi}(x_4, x_3) \\\\
z_9 &= 0 + 2 \mathrm{hi}(x_4, x_4) \\\\
\end{aligned}
\\]
To implement these, we group terms by their coefficient, computing
those with coefficient \\(2\\) on set of IFMA chains, and on another
set of chains, we begin with coefficient-\\(4\\) terms, then shift
left before continuing with the coefficient-\\(1\\) terms.
The reduction strategy is the same as for multiplication.

# Future improvements

LLVM won't use blend operations on [256-bit vectors yet][llvm_blend],
so there's a bunch of blend instructions that could be omitted.

Although the multiplications and squarings are much faster, there's no
speedup to the additions and subtractions, so there are diminishing
returns.  In fact, the complications in the doubling formulas mean
that doubling is actually slower than readdition.  This also suggests
that moving to 512-bit vectors won't be much help for a strategy aimed
at parallelism within a group operation, so to extract performance
gains from 512-bit vectors it will probably be necessary to create a
parallel-friendly multiscalar multiplication algorithm.  This could
also help with reducing shuffle pressure.

The squaring implementation could probably be optimized, but without
`perf` support on Cannonlake it's difficult to make actual
measurements.

Another improvement would be to implement vectorized square root
computations, which would allow creating an iterator adaptor for point
decompression that bunched decompression operations and executed them
in parallel.  This would accelerate batch verification.

[2016_gueron_krasnov]: https://ieeexplore.ieee.org/document/7563269
[2018_drucker_gueron]: https://eprint.iacr.org/2018/335
[1999_walter]: https://pdfs.semanticscholar.org/0e6a/3e8f30b63b556679f5dff2cbfdfe9523f4fa.pdf
[ed25519_paper]: https://ed25519.cr.yp.to/ed25519-20110926.pdf
[llvm_blend]: https://bugs.llvm.org/show_bug.cgi?id=38343
