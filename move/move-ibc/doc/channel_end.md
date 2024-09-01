
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::channel`



-  [Struct `Channel`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel)
-  [Struct `Counterparty`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty)
-  [Constants](#@Constants_0)
-  [Function `state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_state)
-  [Function `ordering`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_ordering)
-  [Function `chan_counterparty_port_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_chan_counterparty_port_id)
-  [Function `chan_counterparty_channel_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_chan_counterparty_channel_id)
-  [Function `connection_hops`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_connection_hops)
-  [Function `version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_version)
-  [Function `counterparty_port_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_counterparty_port_id)
-  [Function `counterparty_channel_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_counterparty_channel_id)
-  [Function `set_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_state)
-  [Function `set_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_version)
-  [Function `set_ordering`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_ordering)
-  [Function `set_chan_counterparty_channel_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_chan_counterparty_channel_id)
-  [Function `new`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_new)
-  [Function `default`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_default)
-  [Function `new_counterparty`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_new_counterparty)
-  [Function `default_counterparty`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_default_counterparty)
-  [Function `encode_proto`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_encode_proto)
-  [Function `encode_proto_counterparty`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_encode_proto_counterparty)
-  [Function `decode_proto`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_decode_proto)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::proto_utils</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel"></a>

## Struct `Channel`



<pre><code><b>struct</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">Channel</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty"></a>

## Struct `Counterparty`



<pre><code><b>struct</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">Counterparty</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_ORDERING_NONE"></a>



<pre><code><b>const</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_ORDERING_NONE">CHAN_ORDERING_NONE</a>: u8 = 0;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_ORDERING_ORDERED"></a>



<pre><code><b>const</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_ORDERING_ORDERED">CHAN_ORDERING_ORDERED</a>: u8 = 2;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_ORDERING_UNORDERED"></a>



<pre><code><b>const</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_ORDERING_UNORDERED">CHAN_ORDERING_UNORDERED</a>: u8 = 1;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_CLOSED"></a>



<pre><code><b>const</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_CLOSED">CHAN_STATE_CLOSED</a>: u8 = 4;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_INIT"></a>



<pre><code><b>const</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_INIT">CHAN_STATE_INIT</a>: u8 = 1;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_OPEN"></a>



<pre><code><b>const</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_OPEN">CHAN_STATE_OPEN</a>: u8 = 3;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_TRYOPEN"></a>



<pre><code><b>const</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_TRYOPEN">CHAN_STATE_TRYOPEN</a>: u8 = 2;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_UNINITIALIZED"></a>



<pre><code><b>const</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_CHAN_STATE_UNINITIALIZED">CHAN_STATE_UNINITIALIZED</a>: u8 = 0;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_state"></a>

## Function `state`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_state">state</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>): u8
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_ordering"></a>

## Function `ordering`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_ordering">ordering</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>): u8
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_chan_counterparty_port_id"></a>

## Function `chan_counterparty_port_id`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_chan_counterparty_port_id">chan_counterparty_port_id</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_chan_counterparty_channel_id"></a>

## Function `chan_counterparty_channel_id`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_chan_counterparty_channel_id">chan_counterparty_channel_id</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_connection_hops"></a>

## Function `connection_hops`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_connection_hops">connection_hops</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_version"></a>

## Function `version`



<pre><code><b>public</b> <b>fun</b> <a href="">version</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_counterparty_port_id"></a>

## Function `counterparty_port_id`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_counterparty_port_id">counterparty_port_id</a>(counterparty: &<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">channel::Counterparty</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_counterparty_channel_id"></a>

## Function `counterparty_channel_id`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_counterparty_channel_id">counterparty_channel_id</a>(counterparty: &<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">channel::Counterparty</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_state"></a>

## Function `set_state`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_state">set_state</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<b>mut</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>, state: u8)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_version"></a>

## Function `set_version`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_version">set_version</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<b>mut</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>, <a href="">version</a>: <a href="_String">string::String</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_ordering"></a>

## Function `set_ordering`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_ordering">set_ordering</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<b>mut</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>, ordering: u8)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_chan_counterparty_channel_id"></a>

## Function `set_chan_counterparty_channel_id`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_set_chan_counterparty_channel_id">set_chan_counterparty_channel_id</a>(<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: &<b>mut</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>, channel_id: <a href="_String">string::String</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_new">new</a>(state: u8, ordering: u8, counterparty: <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">channel::Counterparty</a>, connection_hops: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">version</a>: <a href="_String">string::String</a>): <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_default"></a>

## Function `default`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_default">default</a>(): <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_new_counterparty"></a>

## Function `new_counterparty`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_new_counterparty">new_counterparty</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">channel::Counterparty</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_default_counterparty"></a>

## Function `default_counterparty`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_default_counterparty">default_counterparty</a>(): <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">channel::Counterparty</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_encode_proto"></a>

## Function `encode_proto`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_encode_proto">encode_proto</a>(chan: <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_encode_proto_counterparty"></a>

## Function `encode_proto_counterparty`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_encode_proto_counterparty">encode_proto_counterparty</a>(value: <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">channel::Counterparty</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_decode_proto"></a>

## Function `decode_proto`



<pre><code><b>public</b> <b>fun</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_decode_proto">decode_proto</a>(buf: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>&gt;
</code></pre>
