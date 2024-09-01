
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::height`



-  [Resource `Height`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height)
-  [Function `new`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_new)
-  [Function `default`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_default)
-  [Function `get_revision_number`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_get_revision_number)
-  [Function `get_revision_height`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_get_revision_height)
-  [Function `is_zero`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_is_zero)
-  [Function `gte`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_gte)
-  [Function `set_revision_height`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_set_revision_height)
-  [Function `encode_proto`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_encode_proto)
-  [Function `decode_proto`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_decode_proto)


<pre><code><b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::proto_utils</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height"></a>

## Resource `Height`



<pre><code><b>struct</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">Height</a> <b>has</b> <b>copy</b>, drop, store, key
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_new">new</a>(revision_number: u64, revision_height: u64): <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_default"></a>

## Function `default`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_default">default</a>(): <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_get_revision_number"></a>

## Function `get_revision_number`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_get_revision_number">get_revision_number</a>(<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: &<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_get_revision_height"></a>

## Function `get_revision_height`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_get_revision_height">get_revision_height</a>(<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: &<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_is_zero"></a>

## Function `is_zero`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_is_zero">is_zero</a>(<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: &<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_gte"></a>

## Function `gte`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_gte">gte</a>(height1: &<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, height2: &<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_set_revision_height"></a>

## Function `set_revision_height`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_set_revision_height">set_revision_height</a>(<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: &<b>mut</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, revision_height: u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_encode_proto"></a>

## Function `encode_proto`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_encode_proto">encode_proto</a>(<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_decode_proto"></a>

## Function `decode_proto`



<pre><code><b>public</b> <b>fun</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_decode_proto">decode_proto</a>(buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64, len: u64, <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: &<b>mut</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): (u64, u64)
</code></pre>
