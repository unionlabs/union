
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::IBCCommitment`



-  [Function `keccak256`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_keccak256)
-  [Function `client_state_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_client_state_path)
-  [Function `consensus_state_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_consensus_state_path)
-  [Function `connection_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_connection_path)
-  [Function `channel_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_path)
-  [Function `packet_commitment_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_commitment_path)
-  [Function `packet_acknowledgement_commitment_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_acknowledgement_commitment_path)
-  [Function `packet_receipt_commitment_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_receipt_commitment_path)
-  [Function `next_sequence_send_commitment_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_send_commitment_path)
-  [Function `next_sequence_recv_commitment_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_recv_commitment_path)
-  [Function `channel_capability_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_capability_path)
-  [Function `next_sequence_ack_commitment_path`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_ack_commitment_path)
-  [Function `client_state_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_client_state_commitment_key)
-  [Function `consensus_state_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_consensus_state_commitment_key)
-  [Function `connection_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_connection_commitment_key)
-  [Function `channel_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_commitment_key)
-  [Function `packet_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_commitment_key)
-  [Function `packet_acknowledgement_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_acknowledgement_commitment_key)
-  [Function `packet_receipt_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_receipt_commitment_key)
-  [Function `next_sequence_send_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_send_commitment_key)
-  [Function `next_sequence_recv_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_recv_commitment_key)
-  [Function `next_sequence_ack_commitment_key`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_ack_commitment_key)


<pre><code><b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::string_utils</a>;
<b>use</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::height</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_keccak256"></a>

## Function `keccak256`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_keccak256">keccak256</a>(s: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_client_state_path"></a>

## Function `client_state_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_client_state_path">client_state_path</a>(client_id: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_consensus_state_path"></a>

## Function `consensus_state_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_consensus_state_path">consensus_state_path</a>(client_id: <a href="_String">string::String</a>, revision_number: u64, revision_height: u64): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_connection_path"></a>

## Function `connection_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_connection_path">connection_path</a>(connection_id: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_path"></a>

## Function `channel_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_path">channel_path</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_commitment_path"></a>

## Function `packet_commitment_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_commitment_path">packet_commitment_path</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, sequence: u64): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_acknowledgement_commitment_path"></a>

## Function `packet_acknowledgement_commitment_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_acknowledgement_commitment_path">packet_acknowledgement_commitment_path</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, sequence: u64): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_receipt_commitment_path"></a>

## Function `packet_receipt_commitment_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_receipt_commitment_path">packet_receipt_commitment_path</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, sequence: u64): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_send_commitment_path"></a>

## Function `next_sequence_send_commitment_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_send_commitment_path">next_sequence_send_commitment_path</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_recv_commitment_path"></a>

## Function `next_sequence_recv_commitment_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_recv_commitment_path">next_sequence_recv_commitment_path</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_capability_path"></a>

## Function `channel_capability_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_capability_path">channel_capability_path</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_ack_commitment_path"></a>

## Function `next_sequence_ack_commitment_path`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_ack_commitment_path">next_sequence_ack_commitment_path</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_client_state_commitment_key"></a>

## Function `client_state_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_client_state_commitment_key">client_state_commitment_key</a>(client_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_consensus_state_commitment_key"></a>

## Function `consensus_state_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_consensus_state_commitment_key">consensus_state_commitment_key</a>(client_id: <a href="_String">string::String</a>, <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_connection_commitment_key"></a>

## Function `connection_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_connection_commitment_key">connection_commitment_key</a>(connection_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_commitment_key"></a>

## Function `channel_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_channel_commitment_key">channel_commitment_key</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_commitment_key"></a>

## Function `packet_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_commitment_key">packet_commitment_key</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, sequence: u64): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_acknowledgement_commitment_key"></a>

## Function `packet_acknowledgement_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_acknowledgement_commitment_key">packet_acknowledgement_commitment_key</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, sequence: u64): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_receipt_commitment_key"></a>

## Function `packet_receipt_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_packet_receipt_commitment_key">packet_receipt_commitment_key</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, sequence: u64): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_send_commitment_key"></a>

## Function `next_sequence_send_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_send_commitment_key">next_sequence_send_commitment_key</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_recv_commitment_key"></a>

## Function `next_sequence_recv_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_recv_commitment_key">next_sequence_recv_commitment_key</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_ack_commitment_key"></a>

## Function `next_sequence_ack_commitment_key`



<pre><code><b>public</b> <b>fun</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment_next_sequence_ack_commitment_key">next_sequence_ack_commitment_key</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>
