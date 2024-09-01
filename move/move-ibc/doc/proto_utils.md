
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::proto_utils`



-  [Function `encode_string`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_string)
-  [Function `encode_bytes`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_bytes)
-  [Function `encode_varint`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_varint)
-  [Function `decode_string`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_string)
-  [Function `decode_bytes`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_bytes)
-  [Function `decode_untagged_bytes`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_untagged_bytes)
-  [Function `decode_untagged_string`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_untagged_string)
-  [Function `decode_prefix`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_prefix)
-  [Function `decode_nested_len`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_nested_len)
-  [Function `decode_varint`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_varint)
-  [Function `decode_varint_raw`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_varint_raw)
-  [Function `encode_prefix`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_prefix)
-  [Function `encode_u64`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_u64)
-  [Function `encode_u32`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_u32)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_string"></a>

## Function `encode_string`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_string">encode_string</a>(field: u8, value: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_bytes"></a>

## Function `encode_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_bytes">encode_bytes</a>(field: u8, value: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_varint"></a>

## Function `encode_varint`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_varint">encode_varint</a>(value: u64): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_string"></a>

## Function `decode_string`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_string">decode_string</a>(wire_type: u64, buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64): (<a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_bytes"></a>

## Function `decode_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_bytes">decode_bytes</a>(wire_type: u64, buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64): (<a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_untagged_bytes"></a>

## Function `decode_untagged_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_untagged_bytes">decode_untagged_bytes</a>(buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64): (<a href="_Option">option::Option</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt;, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_untagged_string"></a>

## Function `decode_untagged_string`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_untagged_string">decode_untagged_string</a>(buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64): (<a href="_Option">option::Option</a>&lt;<a href="_String">string::String</a>&gt;, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_prefix"></a>

## Function `decode_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_prefix">decode_prefix</a>(buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64): (u64, u64, u64, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_nested_len"></a>

## Function `decode_nested_len`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_nested_len">decode_nested_len</a>(wire_type: u64, buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64): (u64, u64, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_varint"></a>

## Function `decode_varint`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_varint">decode_varint</a>(wire_type: u64, buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64): (u64, u64, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_varint_raw"></a>

## Function `decode_varint_raw`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_decode_varint_raw">decode_varint_raw</a>(buf: &<a href="">vector</a>&lt;u8&gt;, cursor: u64): (u64, u64, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_prefix"></a>

## Function `encode_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_prefix">encode_prefix</a>(field: u8, wire_type: u8): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_u64"></a>

## Function `encode_u64`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_u64">encode_u64</a>(field: u8, value: u64): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_u32"></a>

## Function `encode_u32`



<pre><code><b>public</b> <b>fun</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils_encode_u32">encode_u32</a>(field: u8, value: u32): <a href="">vector</a>&lt;u8&gt;
</code></pre>
