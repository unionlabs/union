
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::ics23`



-  [Struct `MembershipProof`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_MembershipProof)
-  [Struct `ExistenceProof`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_ExistenceProof)
-  [Struct `InnerOp`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_InnerOp)
-  [Struct `NonExistenceProof`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_NonExistenceProof)
-  [Struct `ProofSpec`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_ProofSpec)
-  [Function `verify_membership`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_verify_membership)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_MembershipProof"></a>

## Struct `MembershipProof`



<pre><code><b>struct</b> <a href="ics23.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_MembershipProof">MembershipProof</a> <b>has</b> drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_ExistenceProof"></a>

## Struct `ExistenceProof`



<pre><code><b>struct</b> <a href="ics23.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_ExistenceProof">ExistenceProof</a> <b>has</b> drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_InnerOp"></a>

## Struct `InnerOp`



<pre><code><b>struct</b> <a href="ics23.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_InnerOp">InnerOp</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_NonExistenceProof"></a>

## Struct `NonExistenceProof`



<pre><code><b>struct</b> <a href="ics23.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_NonExistenceProof">NonExistenceProof</a> <b>has</b> drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_ProofSpec"></a>

## Struct `ProofSpec`



<pre><code><b>struct</b> <a href="ics23.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_ProofSpec">ProofSpec</a> <b>has</b> drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_verify_membership"></a>

## Function `verify_membership`



<pre><code><b>public</b> <b>fun</b> <a href="ics23.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_verify_membership">verify_membership</a>(proof: <a href="ics23.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_ics23_MembershipProof">ics23::MembershipProof</a>, root: <a href="">vector</a>&lt;u8&gt;, prefix: <a href="">vector</a>&lt;u8&gt;, key: <a href="">vector</a>&lt;u8&gt;, value: <a href="">vector</a>&lt;u8&gt;): u64
</code></pre>
