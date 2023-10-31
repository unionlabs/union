An AVX2 implementation of the vectorized point operation strategy.

# Field element representation

Our strategy is to implement 4-wide multiplication and squaring by
wordslicing, using one 64-bit AVX2 lane for each field element.  Field
elements are represented in the usual way as 10 `u32` limbs in radix
\\(25.5\\) (i.e., alternating between \\(2\^{26}\\) for even limbs and
\\(2\^{25}\\) for odd limbs).  This has the effect that passing between
the parallel 32-bit AVX2 representation and the serial 64-bit
representation (which uses radix \\(2^{51}\\)) amounts to regrouping
digits.

The field element representation is oriented around the AVX2
`vpmuludq` instruction, which multiplies the low 32 bits of each
64-bit lane of each operand to produce a 64-bit result.

```text,no_run
(a1 ?? b1 ?? c1 ?? d1 ??)
(a2 ?? b2 ?? c2 ?? d2 ??)

(a1*a2 b1*b2 c1*c2 d1*d2)
```

To unpack 32-bit values into 64-bit lanes for use in multiplication
it would be convenient to use the `vpunpck[lh]dq` instructions,
which unpack and interleave the low and high 32-bit lanes of two
source vectors.
However, the AVX2 versions of these instructions are designed to
operate only within 128-bit lanes of the 256-bit vectors, so that
interleaving the low lanes of `(a0 b0 c0 d0 a1 b1 c1 d1)` with zero
gives `(a0 00 b0 00 a1 00 b1 00)`.  Instead, we pre-shuffle the data
layout as `(a0 b0 a1 b1 c0 d0 c1 d1)` so that we can unpack the
"low" and "high" parts as

```text,no_run
(a0 00 b0 00 c0 00 d0 00)
(a1 00 b1 00 c1 00 d1 00)
```

The data layout for a vector of four field elements \\( (a,b,c,d)
\\) with limbs \\( a_0, a_1, \ldots, a_9 \\) is as `[u32x8; 5]` in
the form

```text,no_run
(a0 b0 a1 b1 c0 d0 c1 d1)
(a2 b2 a3 b3 c2 d2 c3 d3)
(a4 b4 a5 b5 c4 d4 c5 d5)
(a6 b6 a7 b7 c6 d6 c7 d7)
(a8 b8 a9 b9 c8 d8 c9 d9)
```

Since this breaks cleanly into two 128-bit lanes, it may be possible
to adapt it to 128-bit vector instructions such as NEON without too
much difficulty.

# Avoiding Overflow in Doubling

To analyze the size of the field element coefficients during the
computations, we can parameterize the bounds on the limbs of each
field element by \\( b \in \mathbb R \\) representing the excess bits
above that limb's radix, so that each limb is bounded by either
\\(2\^{25+b} \\) or \\( 2\^{26+b} \\), as appropriate.

The multiplication routine requires that its inputs are bounded with
\\( b < 1.75 \\), in order to fit a multiplication by \\( 19 \\)
into 32 bits.  Since \\( \lg 19 < 4.25 \\), \\( 19x < 2\^{32} \\)
when \\( x < 2\^{27.75} = 2\^{26 + 1.75} \\).  However, this is only
required for one of the inputs; the other can grow up to \\( b < 2.5
\\).

In addition, the multiplication and squaring routines do not
canonically reduce their outputs, but can leave some small uncarried
excesses, so that their reduced outputs are bounded with
\\( b < 0.007 \\).

The non-parallel portion of the doubling formulas is
$$
\begin{aligned}
(S\_5 &&,&& S\_6 &&,&& S\_8 &&,&& S\_9 )
&\gets
(S\_1 + S\_2 &&,&& S\_1 - S\_2 &&,&& S\_1 + 2S\_3 - S\_2 &&,&& S\_1 + S\_2 - S\_4)
\end{aligned}
$$

Computing \\( (S\_5, S\_6, S\_8, S\_9 ) \\) as
$$
\begin{matrix}
 & S\_1 & S\_1 & S\_1 & S\_1 \\\\
+& S\_2 &      &      & S\_2 \\\\
+&      &      & S\_3 &      \\\\
+&      &      & S\_3 &      \\\\
+&      & 2p   & 2p   & 2p   \\\\
-&      & S\_2 & S\_2 &      \\\\
-&      &      &      & S\_4 \\\\
=& S\_5 & S\_6 & S\_8 & S\_9
\end{matrix}
$$
results in bit-excesses \\( < (1.01, 1.60, 2.33, 2.01)\\) for
\\( (S\_5, S\_6, S\_8, S\_9 ) \\).  The products we want to compute
are then
$$
\begin{aligned}
X\_3 &\gets S\_8 S\_9 \leftrightarrow (2.33, 2.01) \\\\
Y\_3 &\gets S\_5 S\_6 \leftrightarrow (1.01, 1.60) \\\\
Z\_3 &\gets S\_8 S\_6 \leftrightarrow (2.33, 1.60) \\\\
T\_3 &\gets S\_5 S\_9 \leftrightarrow (1.01, 2.01)
\end{aligned}
$$
which are too large: it's not possible to arrange the multiplicands so
that one vector has \\(b < 2.5\\) and the other has \\( b < 1.75 \\).
However, if we flip the sign of \\( S\_4 = S\_0\^2 \\) during
squaring, so that we output \\(S\_4' = -S\_4 \pmod p\\), then we can
compute
$$
\begin{matrix}
 & S\_1 & S\_1 & S\_1 & S\_1 \\\\
+& S\_2 &      &      & S\_2 \\\\
+&      &      & S\_3 &      \\\\
+&      &      & S\_3 &      \\\\
+&      &      &      & S\_4' \\\\
+&      & 2p   & 2p   &      \\\\
-&      & S\_2 & S\_2 &      \\\\
=& S\_5 & S\_6 & S\_8 & S\_9
\end{matrix}
$$
resulting in bit-excesses \\( < (1.01, 1.60, 2.33, 1.60)\\) for
\\( (S\_5, S\_6, S\_8, S\_9 ) \\).  The products we want to compute
are then
$$
\begin{aligned}
X\_3 &\gets S\_8 S\_9 \leftrightarrow (2.33, 1.60) \\\\
Y\_3 &\gets S\_5 S\_6 \leftrightarrow (1.01, 1.60) \\\\
Z\_3 &\gets S\_8 S\_6 \leftrightarrow (2.33, 1.60) \\\\
T\_3 &\gets S\_5 S\_9 \leftrightarrow (1.01, 1.60)
\end{aligned}
$$
whose right-hand sides are all bounded with \\( b < 1.75 \\) and
whose left-hand sides are all bounded with \\( b < 2.5 \\),
so that we can avoid any intermediate reductions.
