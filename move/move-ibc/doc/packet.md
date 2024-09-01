
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::packet`



-  [Resource `Packet`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet)
-  [Function `sequence`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_sequence)
-  [Function `source_port`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_source_port)
-  [Function `source_channel`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_source_channel)
-  [Function `destination_port`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_destination_port)
-  [Function `destination_channel`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_destination_channel)
-  [Function `data`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_data)
-  [Function `timeout_height`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_timeout_height)
-  [Function `timeout_timestamp`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_timeout_timestamp)
-  [Function `commitment`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_commitment)
-  [Function `commitment_from_parts`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_commitment_from_parts)
-  [Function `new`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_new)
-  [Function `default`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_default)
-  [Function `encode_proto`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_encode_proto)
-  [Function `decode_proto`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_decode_proto)


<pre><code><b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::height</a>;
<b>use</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::proto_utils</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet"></a>

## Resource `Packet`



<pre><code><b>struct</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">Packet</a> <b>has</b> <b>copy</b>, drop, store, key
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_sequence"></a>

## Function `sequence`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_sequence">sequence</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_source_port"></a>

## Function `source_port`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_source_port">source_port</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_source_channel"></a>

## Function `source_channel`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_source_channel">source_channel</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_destination_port"></a>

## Function `destination_port`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_destination_port">destination_port</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_destination_channel"></a>

## Function `destination_channel`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_destination_channel">destination_channel</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_data"></a>

## Function `data`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_data">data</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_timeout_height"></a>

## Function `timeout_height`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_timeout_height">timeout_height</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_timeout_timestamp"></a>

## Function `timeout_timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_timeout_timestamp">timeout_timestamp</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_commitment"></a>

## Function `commitment`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_commitment">commitment</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: &<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_commitment_from_parts"></a>

## Function `commitment_from_parts`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_commitment_from_parts">commitment_from_parts</a>(timeout_timestamp: u64, timeout_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, data: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_new">new</a>(sequence: u64, source_port: <a href="_String">string::String</a>, source_channel: <a href="_String">string::String</a>, destination_port: <a href="_String">string::String</a>, destination_channel: <a href="_String">string::String</a>, data: <a href="">vector</a>&lt;u8&gt;, timeout_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, timeout_timestamp: u64): <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_default"></a>

## Function `default`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_default">default</a>(): <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_encode_proto"></a>

## Function `encode_proto`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_encode_proto">encode_proto</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_decode_proto"></a>

## Function `decode_proto`



<pre><code><b>public</b> <b>fun</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_decode_proto">decode_proto</a>(buf: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>&gt;
</code></pre>
