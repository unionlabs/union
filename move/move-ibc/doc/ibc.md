
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::Core`



-  [Struct `ClientCreatedEvent`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ClientCreatedEvent)
-  [Struct `ConnectionOpenInit`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenInit)
-  [Struct `ChannelOpenInit`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenInit)
-  [Struct `ChannelOpenTry`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenTry)
-  [Struct `ChannelOpenAck`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenAck)
-  [Struct `ChannelOpenConfirm`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenConfirm)
-  [Struct `ConnectionOpenTry`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenTry)
-  [Struct `ConnectionOpenAck`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenAck)
-  [Struct `ConnectionOpenConfirm`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenConfirm)
-  [Struct `SendPacket`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_SendPacket)
-  [Struct `RecvPacket`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_RecvPacket)
-  [Struct `TimeoutPacket`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_TimeoutPacket)
-  [Struct `WriteAcknowledgement`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_WriteAcknowledgement)
-  [Struct `AcknowledgePacket`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_AcknowledgePacket)
-  [Struct `ChannelPort`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelPort)
-  [Resource `IBCStore`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_IBCStore)
-  [Resource `SignerRef`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_SignerRef)
-  [Constants](#@Constants_0)
-  [Function `set_capability`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_capability)
-  [Function `client_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_client_state)
-  [Function `consensus_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_consensus_state)
-  [Function `get_vault_addr`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_vault_addr)
-  [Function `set_connection`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_connection)
-  [Function `set_channel`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_channel)
-  [Function `set_commitment`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_commitment)
-  [Function `get_commitment`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_commitment)
-  [Function `get_channel_from_store`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_channel_from_store)
-  [Function `get_capability_from_store`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_capability_from_store)
-  [Function `set_next_channel_sequence`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_next_channel_sequence)
-  [Function `get_next_channel_sequence`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_next_channel_sequence)
-  [Function `hackerman`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_hackerman)
-  [Function `generate_client_identifier`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_client_identifier)
-  [Function `create_client`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_create_client)
-  [Function `get_ibc_signer`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_ibc_signer)
-  [Function `default_ibc_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_default_ibc_version)
-  [Function `set_supported_versions`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_supported_versions)
-  [Function `is_supported_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_is_supported_version)
-  [Function `get_feature_set_intersection`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_feature_set_intersection)
-  [Function `pick_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_pick_version)
-  [Function `copy_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_copy_version)
-  [Function `copy_versions`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_copy_versions)
-  [Function `find_supported_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_find_supported_version)
-  [Function `verify_proposed_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_proposed_version)
-  [Function `verify_client_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_client_state)
-  [Function `verify_connection_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_connection_state)
-  [Function `verify_commitment`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_commitment)
-  [Function `generate_connection_identifier`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_connection_identifier)
-  [Function `update_connection_commitment`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_update_connection_commitment)
-  [Function `connection_open_init`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_init)
-  [Function `get_compatible_versions`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_compatible_versions)
-  [Function `get_connection`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_connection)
-  [Function `get_connection_commitment`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_connection_commitment)
-  [Function `connection_open_try`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_try)
-  [Function `connection_open_ack`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_ack)
-  [Function `connection_open_confirm`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_confirm)
-  [Function `verify_supported_feature`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_supported_feature)
-  [Function `to_string`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_to_string)
-  [Function `get_counterparty_hops`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_counterparty_hops)
-  [Function `generate_channel_identifier`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_channel_identifier)
-  [Function `ensure_connection_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_connection_state)
-  [Function `ensure_connection_feature`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_connection_feature)
-  [Function `is_lowercase`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_is_lowercase)
-  [Function `update_channel_commitment`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_update_channel_commitment)
-  [Function `verify_channel_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_channel_state)
-  [Function `claim_capability`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_claim_capability)
-  [Function `create_new_table`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_create_new_table)
-  [Function `channel_open_init`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_init)
-  [Function `channel_open_ack`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_ack)
-  [Function `channel_open_confirm`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_confirm)
-  [Function `channel_open_try`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_try)
-  [Function `ensure_channel_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_channel_state)
-  [Function `authenticate_capability`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_authenticate_capability)
-  [Function `send_packet`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_send_packet)
-  [Function `recv_packet`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_recv_packet)
-  [Function `write_acknowledgement`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_write_acknowledgement)
-  [Function `write_ack_impl`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_write_ack_impl)
-  [Function `acknowledge_packet`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_acknowledge_packet)
-  [Function `timeout_packet`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_timeout_packet)
-  [Function `verify_absent_commitment`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_absent_commitment)


<pre><code><b>use</b> <a href="">0x1::any</a>;
<b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="">0x1::debug</a>;
<b>use</b> <a href="">0x1::event</a>;
<b>use</b> <a href="">0x1::from_bcs</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::object</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::smart_table</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::string_utils</a>;
<b>use</b> <a href="">0x1::table</a>;
<b>use</b> <a href="">0x1::timestamp</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="ibc_commitment.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_IBCCommitment">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::IBCCommitment</a>;
<b>use</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::LightClient</a>;
<b>use</b> <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::channel</a>;
<b>use</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::connection_end</a>;
<b>use</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::height</a>;
<b>use</b> <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::packet</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ClientCreatedEvent"></a>

## Struct `ClientCreatedEvent`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ClientCreatedEvent">ClientCreatedEvent</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenInit"></a>

## Struct `ConnectionOpenInit`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenInit">ConnectionOpenInit</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenInit"></a>

## Struct `ChannelOpenInit`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenInit">ChannelOpenInit</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenTry"></a>

## Struct `ChannelOpenTry`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenTry">ChannelOpenTry</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenAck"></a>

## Struct `ChannelOpenAck`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenAck">ChannelOpenAck</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenConfirm"></a>

## Struct `ChannelOpenConfirm`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelOpenConfirm">ChannelOpenConfirm</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenTry"></a>

## Struct `ConnectionOpenTry`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenTry">ConnectionOpenTry</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenAck"></a>

## Struct `ConnectionOpenAck`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenAck">ConnectionOpenAck</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenConfirm"></a>

## Struct `ConnectionOpenConfirm`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ConnectionOpenConfirm">ConnectionOpenConfirm</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_SendPacket"></a>

## Struct `SendPacket`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_SendPacket">SendPacket</a> <b>has</b> drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_RecvPacket"></a>

## Struct `RecvPacket`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_RecvPacket">RecvPacket</a> <b>has</b> drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_TimeoutPacket"></a>

## Struct `TimeoutPacket`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_TimeoutPacket">TimeoutPacket</a> <b>has</b> drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_WriteAcknowledgement"></a>

## Struct `WriteAcknowledgement`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_WriteAcknowledgement">WriteAcknowledgement</a> <b>has</b> drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_AcknowledgePacket"></a>

## Struct `AcknowledgePacket`



<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_AcknowledgePacket">AcknowledgePacket</a> <b>has</b> drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelPort"></a>

## Struct `ChannelPort`



<pre><code><b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ChannelPort">ChannelPort</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_IBCStore"></a>

## Resource `IBCStore`



<pre><code><b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_IBCStore">IBCStore</a> <b>has</b> key
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_SignerRef"></a>

## Resource `SignerRef`



<pre><code><b>struct</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_SignerRef">SignerRef</a> <b>has</b> key
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_ORDERING_NONE"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_ORDERING_NONE">CHAN_ORDERING_NONE</a>: u8 = 0;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_ORDERING_ORDERED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_ORDERING_ORDERED">CHAN_ORDERING_ORDERED</a>: u8 = 2;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_ORDERING_UNORDERED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_ORDERING_UNORDERED">CHAN_ORDERING_UNORDERED</a>: u8 = 1;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_CLOSED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_CLOSED">CHAN_STATE_CLOSED</a>: u8 = 4;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_INIT"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_INIT">CHAN_STATE_INIT</a>: u8 = 1;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_OPEN"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_OPEN">CHAN_STATE_OPEN</a>: u8 = 3;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_TRYOPEN"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_TRYOPEN">CHAN_STATE_TRYOPEN</a>: u8 = 2;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_UNINITIALIZED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CHAN_STATE_UNINITIALIZED">CHAN_STATE_UNINITIALIZED</a>: u8 = 0;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CONN_STATE_INIT"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CONN_STATE_INIT">CONN_STATE_INIT</a>: u64 = 1;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CONN_STATE_OPEN"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CONN_STATE_OPEN">CONN_STATE_OPEN</a>: u64 = 3;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CONN_STATE_TRYOPEN"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CONN_STATE_TRYOPEN">CONN_STATE_TRYOPEN</a>: u64 = 2;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CONN_STATE_UNSPECIFIED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_CONN_STATE_UNSPECIFIED">CONN_STATE_UNSPECIFIED</a>: u64 = 0;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_ACKNOWLEDGEMENT_ALREADY_EXISTS"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_ACKNOWLEDGEMENT_ALREADY_EXISTS">E_ACKNOWLEDGEMENT_ALREADY_EXISTS</a>: u64 = 1029;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_ACKNOWLEDGEMENT_IS_EMPTY"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_ACKNOWLEDGEMENT_IS_EMPTY">E_ACKNOWLEDGEMENT_IS_EMPTY</a>: u64 = 1028;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CAPABILITY_ALREADY_CLAIMED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CAPABILITY_ALREADY_CLAIMED">E_CAPABILITY_ALREADY_CLAIMED</a>: u64 = 1014;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CLIENT_ALREADY_EXISTS"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CLIENT_ALREADY_EXISTS">E_CLIENT_ALREADY_EXISTS</a>: u64 = 1001;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CLIENT_IMPL_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CLIENT_IMPL_NOT_FOUND">E_CLIENT_IMPL_NOT_FOUND</a>: u64 = 1002;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CONNECTION_ALREADY_EXISTS"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CONNECTION_ALREADY_EXISTS">E_CONNECTION_ALREADY_EXISTS</a>: u64 = 1009;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CONNECTION_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CONNECTION_DOES_NOT_EXIST">E_CONNECTION_DOES_NOT_EXIST</a>: u64 = 1028;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CONN_NOT_SINGLE_HOP"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CONN_NOT_SINGLE_HOP">E_CONN_NOT_SINGLE_HOP</a>: u64 = 1011;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CONN_NOT_SINGLE_VERSION"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_CONN_NOT_SINGLE_VERSION">E_CONN_NOT_SINGLE_VERSION</a>: u64 = 1012;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_COUNTERPARTY_CHANNEL_NOT_EMPTY"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_COUNTERPARTY_CHANNEL_NOT_EMPTY">E_COUNTERPARTY_CHANNEL_NOT_EMPTY</a>: u64 = 1017;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_DESTINATION_AND_COUNTERPARTY_CHANNEL_MISMATCH"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_DESTINATION_AND_COUNTERPARTY_CHANNEL_MISMATCH">E_DESTINATION_AND_COUNTERPARTY_CHANNEL_MISMATCH</a>: u64 = 1031;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_DESTINATION_AND_COUNTERPARTY_PORT_MISMATCH"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_DESTINATION_AND_COUNTERPARTY_PORT_MISMATCH">E_DESTINATION_AND_COUNTERPARTY_PORT_MISMATCH</a>: u64 = 1030;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_HEIGHT_TIMEOUT"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_HEIGHT_TIMEOUT">E_HEIGHT_TIMEOUT</a>: u64 = 1024;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_CHANNEL_STATE"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_CHANNEL_STATE">E_INVALID_CHANNEL_STATE</a>: u64 = 1016;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_CONNECTION_STATE"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_CONNECTION_STATE">E_INVALID_CONNECTION_STATE</a>: u64 = 1008;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_PACKET_COMMITMENT"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_PACKET_COMMITMENT">E_INVALID_PACKET_COMMITMENT</a>: u64 = 1033;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_PROOF"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_PROOF">E_INVALID_PROOF</a>: u64 = 1010;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_TIMEOUT_HEIGHT"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_TIMEOUT_HEIGHT">E_INVALID_TIMEOUT_HEIGHT</a>: u64 = 1018;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_TIMEOUT_TIMESTAMP"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_INVALID_TIMEOUT_TIMESTAMP">E_INVALID_TIMEOUT_TIMESTAMP</a>: u64 = 1021;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_LATEST_HEIGHT_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_LATEST_HEIGHT_NOT_FOUND">E_LATEST_HEIGHT_NOT_FOUND</a>: u64 = 1022;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_LATEST_TIMESTAMP_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_LATEST_TIMESTAMP_NOT_FOUND">E_LATEST_TIMESTAMP_NOT_FOUND</a>: u64 = 1019;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_LIGHT_CLIENT_CALL_FAILED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_LIGHT_CLIENT_CALL_FAILED">E_LIGHT_CLIENT_CALL_FAILED</a>: u64 = 1003;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_NEXT_SEQUENCE_MUST_BE_GREATER_THAN_TIMEOUT_SEQUENCE"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_NEXT_SEQUENCE_MUST_BE_GREATER_THAN_TIMEOUT_SEQUENCE">E_NEXT_SEQUENCE_MUST_BE_GREATER_THAN_TIMEOUT_SEQUENCE</a>: u64 = 1036;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE">E_NOT_ENOUGH_PERMISSIONS_TO_INITIALIZE</a>: u64 = 1005;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_PACKET_ALREADY_RECEIVED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_PACKET_ALREADY_RECEIVED">E_PACKET_ALREADY_RECEIVED</a>: u64 = 1025;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_PACKET_COMMITMENT_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_PACKET_COMMITMENT_NOT_FOUND">E_PACKET_COMMITMENT_NOT_FOUND</a>: u64 = 1032;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH">E_PACKET_SEQUENCE_NEXT_SEQUENCE_MISMATCH</a>: u64 = 1026;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_PORT_ID_MUST_BE_LOWERCASE"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_PORT_ID_MUST_BE_LOWERCASE">E_PORT_ID_MUST_BE_LOWERCASE</a>: u64 = 1015;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_SOURCE_AND_COUNTERPARTY_CHANNEL_MISMATCH"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_SOURCE_AND_COUNTERPARTY_CHANNEL_MISMATCH">E_SOURCE_AND_COUNTERPARTY_CHANNEL_MISMATCH</a>: u64 = 1022;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_SOURCE_AND_COUNTERPARTY_PORT_MISMATCH"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_SOURCE_AND_COUNTERPARTY_PORT_MISMATCH">E_SOURCE_AND_COUNTERPARTY_PORT_MISMATCH</a>: u64 = 1023;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_SWAP_NOT_INITIALIZED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_SWAP_NOT_INITIALIZED">E_SWAP_NOT_INITIALIZED</a>: u64 = 1004;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_TIMEOUT_HEIGHT_NOT_REACHED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_TIMEOUT_HEIGHT_NOT_REACHED">E_TIMEOUT_HEIGHT_NOT_REACHED</a>: u64 = 1035;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_TIMESTAMP_TIMEOUT"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_TIMESTAMP_TIMEOUT">E_TIMESTAMP_TIMEOUT</a>: u64 = 1023;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_TIMESTAMP_TIMEOUT_NOT_REACHED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_TIMESTAMP_TIMEOUT_NOT_REACHED">E_TIMESTAMP_TIMEOUT_NOT_REACHED</a>: u64 = 1034;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_UNAUTHORIZED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_UNAUTHORIZED">E_UNAUTHORIZED</a>: u64 = 1020;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_UNKNOWN_CHANNEL_ORDERING"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_UNKNOWN_CHANNEL_ORDERING">E_UNKNOWN_CHANNEL_ORDERING</a>: u64 = 1027;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_UNSUPPORTED_FEATURE"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_UNSUPPORTED_FEATURE">E_UNSUPPORTED_FEATURE</a>: u64 = 1013;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_UNSUPPORTED_VERSION"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_UNSUPPORTED_VERSION">E_UNSUPPORTED_VERSION</a>: u64 = 1007;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_VERSION_MUST_BE_UNSET"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_E_VERSION_MUST_BE_UNSET">E_VERSION_MUST_BE_UNSET</a>: u64 = 1006;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_SEED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_SEED">SEED</a>: <a href="">vector</a>&lt;u8&gt; = [77, 111, 118, 101, 32, 83, 101, 101, 100, 32, 69, 120, 97, 109, 112, 108, 101];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_VAULT_SEED"></a>



<pre><code><b>const</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_VAULT_SEED">VAULT_SEED</a>: <a href="">vector</a>&lt;u8&gt; = [86, 97, 117, 108, 116, 32, 83, 101, 101, 100, 32, 69, 120, 97, 109, 112, 108, 101];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_capability"></a>

## Function `set_capability`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_capability">set_capability</a>(capability_id: <a href="_String">string::String</a>, addr: <b>address</b>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_client_state"></a>

## Function `client_state`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_client_state">client_state</a>(client_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_consensus_state"></a>

## Function `consensus_state`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_consensus_state">consensus_state</a>(client_id: <a href="_String">string::String</a>, revision_number: u64, revision_height: u64): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_vault_addr"></a>

## Function `get_vault_addr`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_vault_addr">get_vault_addr</a>(): <b>address</b>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_connection"></a>

## Function `set_connection`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_connection">set_connection</a>(connection_id: <a href="_String">string::String</a>, connection: <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_channel"></a>

## Function `set_channel`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_channel">set_channel</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel">channel</a>: <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_commitment"></a>

## Function `set_commitment`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_commitment">set_commitment</a>(key: <a href="">vector</a>&lt;u8&gt;, value: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_commitment"></a>

## Function `get_commitment`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_commitment">get_commitment</a>(key: <a href="">vector</a>&lt;u8&gt;): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_channel_from_store"></a>

## Function `get_channel_from_store`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_channel_from_store">get_channel_from_store</a>(key: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_capability_from_store"></a>

## Function `get_capability_from_store`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_capability_from_store">get_capability_from_store</a>(capability_name: <a href="_String">string::String</a>): <b>address</b>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_next_channel_sequence"></a>

## Function `set_next_channel_sequence`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_next_channel_sequence">set_next_channel_sequence</a>(sequence: u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_next_channel_sequence"></a>

## Function `get_next_channel_sequence`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_next_channel_sequence">get_next_channel_sequence</a>(): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_hackerman"></a>

## Function `hackerman`



<pre><code><b>public</b> entry <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_hackerman">hackerman</a>()
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_client_identifier"></a>

## Function `generate_client_identifier`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_client_identifier">generate_client_identifier</a>(client_type: <a href="_String">string::String</a>): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_create_client"></a>

## Function `create_client`

Create a client with an initial client and consensus state


<pre><code><b>public</b> entry <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_create_client">create_client</a>&lt;CliT: <b>copy</b>, drop, store, ConT: <b>copy</b>, drop, store&gt;(client_type: <a href="_String">string::String</a>, client_state: CliT, consensus_state: ConT)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_ibc_signer"></a>

## Function `get_ibc_signer`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_ibc_signer">get_ibc_signer</a>(): <a href="">signer</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_default_ibc_version"></a>

## Function `default_ibc_version`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_default_ibc_version">default_ibc_version</a>(): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_supported_versions"></a>

## Function `set_supported_versions`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_set_supported_versions">set_supported_versions</a>(supported_versions: <a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;, dst: &<b>mut</b> <a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_is_supported_version"></a>

## Function `is_supported_version`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_is_supported_version">is_supported_version</a>(supported_versions: &<a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;, <a href="">version</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_feature_set_intersection"></a>

## Function `get_feature_set_intersection`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_feature_set_intersection">get_feature_set_intersection</a>(source_features: &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, counterparty_features: &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_pick_version"></a>

## Function `pick_version`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_pick_version">pick_version</a>(supported_versions: &<a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;, counterparty_versions: &<a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_copy_version"></a>

## Function `copy_version`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_copy_version">copy_version</a>(src: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>, dst: &<b>mut</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_copy_versions"></a>

## Function `copy_versions`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_copy_versions">copy_versions</a>(src: &<a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;, dst: &<b>mut</b> <a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_find_supported_version"></a>

## Function `find_supported_version`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_find_supported_version">find_supported_version</a>(supported_versions: &<a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;, <a href="">version</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>): (<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>, bool)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_proposed_version"></a>

## Function `verify_proposed_version`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_proposed_version">verify_proposed_version</a>(supported_version: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>, proposed_version: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_client_state"></a>

## Function `verify_client_state`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_client_state">verify_client_state</a>(connection: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>, <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, path: <a href="">vector</a>&lt;u8&gt;, proof: <a href="">vector</a>&lt;u8&gt;, client_state_bytes: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_connection_state"></a>

## Function `verify_connection_state`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_connection_state">verify_connection_state</a>(connection: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>, <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, proof: <a href="">vector</a>&lt;u8&gt;, connection_id: <a href="_String">string::String</a>, counterparty_connection: <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_commitment"></a>

## Function `verify_commitment`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_commitment">verify_commitment</a>(connection: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>, <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, proof: <a href="">vector</a>&lt;u8&gt;, path: <a href="_String">string::String</a>, commitment: <a href="">vector</a>&lt;u8&gt;): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_connection_identifier"></a>

## Function `generate_connection_identifier`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_connection_identifier">generate_connection_identifier</a>(): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_update_connection_commitment"></a>

## Function `update_connection_commitment`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_update_connection_commitment">update_connection_commitment</a>(store: &<b>mut</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_IBCStore">Core::IBCStore</a>, connection_id: <a href="_String">string::String</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_init"></a>

## Function `connection_open_init`



<pre><code><b>public</b> entry <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_init">connection_open_init</a>(client_id: <a href="_String">string::String</a>, version_identifier: <a href="_String">string::String</a>, version_features: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, counterparty_client_id: <a href="_String">string::String</a>, counterparty_connection_id: <a href="_String">string::String</a>, counterparty_prefix: <a href="">vector</a>&lt;u8&gt;, delay_period: u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_compatible_versions"></a>

## Function `get_compatible_versions`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_compatible_versions">get_compatible_versions</a>(): <a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_connection"></a>

## Function `get_connection`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_connection">get_connection</a>(connection_id: <a href="_String">string::String</a>): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_connection_commitment"></a>

## Function `get_connection_commitment`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_connection_commitment">get_connection_commitment</a>(connection_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_try"></a>

## Function `connection_open_try`



<pre><code><b>public</b> entry <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_try">connection_open_try</a>(counterparty_client_id: <a href="_String">string::String</a>, counterparty_connection_id: <a href="_String">string::String</a>, counterparty_prefix: <a href="">vector</a>&lt;u8&gt;, delay_period: u64, client_id: <a href="_String">string::String</a>, client_state_bytes: <a href="">vector</a>&lt;u8&gt;, counterparty_version_identifiers: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, counterparty_version_features: <a href="">vector</a>&lt;<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;&gt;, proof_init: <a href="">vector</a>&lt;u8&gt;, proof_client: <a href="">vector</a>&lt;u8&gt;, proof_height_revision_num: u64, proof_height_revision_height: u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_ack"></a>

## Function `connection_open_ack`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_ack">connection_open_ack</a>(connection_id: <a href="_String">string::String</a>, client_state_bytes: <a href="">vector</a>&lt;u8&gt;, <a href="">version</a>: <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>, proof_try: <a href="">vector</a>&lt;u8&gt;, proof_client: <a href="">vector</a>&lt;u8&gt;, _proof_consensus: <a href="">vector</a>&lt;u8&gt;, counterparty_connection_id: <a href="_String">string::String</a>, proof_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, _consensus_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_confirm"></a>

## Function `connection_open_confirm`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_connection_open_confirm">connection_open_confirm</a>(connection_id: <a href="_String">string::String</a>, proof_ack: <a href="">vector</a>&lt;u8&gt;, proof_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_supported_feature"></a>

## Function `verify_supported_feature`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_supported_feature">verify_supported_feature</a>(<a href="">version</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>, feature: <a href="_String">string::String</a>): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_to_string"></a>

## Function `to_string`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_to_string">to_string</a>(ordering: u8): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_counterparty_hops"></a>

## Function `get_counterparty_hops`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_get_counterparty_hops">get_counterparty_hops</a>(connection_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_channel_identifier"></a>

## Function `generate_channel_identifier`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_generate_channel_identifier">generate_channel_identifier</a>(): <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_connection_state"></a>

## Function `ensure_connection_state`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_connection_state">ensure_connection_state</a>(connection_id: <a href="_String">string::String</a>): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_connection_feature"></a>

## Function `ensure_connection_feature`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_connection_feature">ensure_connection_feature</a>(connection_hops: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, ordering: u8): (<a href="_String">string::String</a>, <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_is_lowercase"></a>

## Function `is_lowercase`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_is_lowercase">is_lowercase</a>(s: &<a href="_String">string::String</a>): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_update_channel_commitment"></a>

## Function `update_channel_commitment`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_update_channel_commitment">update_channel_commitment</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_channel_state"></a>

## Function `verify_channel_state`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_channel_state">verify_channel_state</a>(connection: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>, <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, proof: <a href="">vector</a>&lt;u8&gt;, port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, channel_bytes: <a href="">vector</a>&lt;u8&gt;): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_claim_capability"></a>

## Function `claim_capability`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_claim_capability">claim_capability</a>(name: <a href="_String">string::String</a>, addr: <b>address</b>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_create_new_table"></a>

## Function `create_new_table`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_create_new_table">create_new_table</a>(): <a href="_SmartTable">smart_table::SmartTable</a>&lt;<a href="_String">string::String</a>, <a href="_SmartTable">smart_table::SmartTable</a>&lt;<a href="_String">string::String</a>, <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>&gt;&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_init"></a>

## Function `channel_open_init`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_init">channel_open_init</a>(ibc_app: &<a href="">signer</a>, port_id: <a href="_String">string::String</a>, connection_hops: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, ordering: u8, counterparty: <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">channel::Counterparty</a>, <a href="">version</a>: <a href="_String">string::String</a>): (<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_ack"></a>

## Function `channel_open_ack`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_ack">channel_open_ack</a>(ibc_app: &<a href="">signer</a>, port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, counterparty_channel_id: <a href="_String">string::String</a>, counterparty_version: <a href="_String">string::String</a>, proof_try: <a href="">vector</a>&lt;u8&gt;, proof_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_confirm"></a>

## Function `channel_open_confirm`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_confirm">channel_open_confirm</a>(ibc_app: &<a href="">signer</a>, port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, proof_ack: <a href="">vector</a>&lt;u8&gt;, proof_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_try"></a>

## Function `channel_open_try`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_channel_open_try">channel_open_try</a>(ibc_app: &<a href="">signer</a>, port_id: <a href="_String">string::String</a>, connection_hops: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, ordering: u8, counterparty: <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Counterparty">channel::Counterparty</a>, counterparty_version: <a href="_String">string::String</a>, <a href="">version</a>: <a href="_String">string::String</a>, proof_init: <a href="">vector</a>&lt;u8&gt;, proof_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): (<a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_channel_state"></a>

## Function `ensure_channel_state`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_ensure_channel_state">ensure_channel_state</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>): <a href="channel_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_channel_Channel">channel::Channel</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_authenticate_capability"></a>

## Function `authenticate_capability`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_authenticate_capability">authenticate_capability</a>(caller: &<a href="">signer</a>, name: <a href="_String">string::String</a>): bool
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_send_packet"></a>

## Function `send_packet`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_send_packet">send_packet</a>(caller: &<a href="">signer</a>, source_channel: <a href="_String">string::String</a>, timeout_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, timeout_timestamp: u64, data: <a href="">vector</a>&lt;u8&gt;): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_recv_packet"></a>

## Function `recv_packet`

Receives and processes an IBC packet

Note that any sanity check failures will result in this function to be aborted in order for caller's
storage to be reverted. This will result in acks won't be able to written.


<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_recv_packet">recv_packet</a>(caller: &<a href="">signer</a>, msg_port_id: <a href="_String">string::String</a>, msg_channel_id: <a href="_String">string::String</a>, msg_packet: <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>, msg_proof: <a href="">vector</a>&lt;u8&gt;, msg_proof_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, acknowledgement: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_write_acknowledgement"></a>

## Function `write_acknowledgement`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_write_acknowledgement">write_acknowledgement</a>(caller: &<a href="">signer</a>, <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>, acknowledgement: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_write_ack_impl"></a>

## Function `write_ack_impl`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_write_ack_impl">write_ack_impl</a>(<a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>, acknowledgement: <a href="">vector</a>&lt;u8&gt;)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_acknowledge_packet"></a>

## Function `acknowledge_packet`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_acknowledge_packet">acknowledge_packet</a>(caller: &<a href="">signer</a>, <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>, acknowledgement: <a href="">vector</a>&lt;u8&gt;, proof: <a href="">vector</a>&lt;u8&gt;, proof_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_timeout_packet"></a>

## Function `timeout_packet`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_timeout_packet">timeout_packet</a>(port_id: <a href="_String">string::String</a>, channel_id: <a href="_String">string::String</a>, <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet">packet</a>: <a href="packet.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_packet_Packet">packet::Packet</a>, proof: <a href="">vector</a>&lt;u8&gt;, proof_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, next_sequence_recv: u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_absent_commitment"></a>

## Function `verify_absent_commitment`



<pre><code><b>public</b> <b>fun</b> <a href="ibc.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_Core_verify_absent_commitment">verify_absent_commitment</a>(connection: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>, <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, proof: <a href="">vector</a>&lt;u8&gt;, path: <a href="_String">string::String</a>): u64
</code></pre>
