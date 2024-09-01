
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::connection_end`



-  [Resource `Version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version)
-  [Struct `ConnectionEnd`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd)
-  [Struct `Counterparty`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Counterparty)
-  [Struct `MerklePrefix`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_MerklePrefix)
-  [Function `new`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new)
-  [Function `new_counterparty`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_counterparty)
-  [Function `new_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_version)
-  [Function `new_versions`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_versions)
-  [Function `delay_period`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_delay_period)
-  [Function `versions`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_versions)
-  [Function `state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_state)
-  [Function `set_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_state)
-  [Function `set_versions`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_versions)
-  [Function `client_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_client_id)
-  [Function `conn_counterparty_client_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_client_id)
-  [Function `conn_counterparty_connection_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_connection_id)
-  [Function `set_conn_counterparty_connection_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_conn_counterparty_connection_id)
-  [Function `conn_counterparty_key_prefix`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_key_prefix)
-  [Function `counterparty_connection_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_counterparty_connection_id)
-  [Function `counterparty_client_id`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_counterparty_client_id)
-  [Function `version_features`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_features)
-  [Function `version_features_mut`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_features_mut)
-  [Function `version_identifier`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_identifier)
-  [Function `version_identifier_mut`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_identifier_mut)
-  [Function `set_version_features`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_version_features)
-  [Function `set_version_identifier`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_version_identifier)
-  [Function `default`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_default)
-  [Function `default_version`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_default_version)
-  [Function `decode_proto`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_decode_proto)
-  [Function `encode_proto`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_encode_proto)


<pre><code><b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="proto_utils.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_proto_utils">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::proto_utils</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version"></a>

## Resource `Version`



<pre><code><b>struct</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">Version</a> <b>has</b> <b>copy</b>, drop, store, key
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd"></a>

## Struct `ConnectionEnd`



<pre><code><b>struct</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">ConnectionEnd</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Counterparty"></a>

## Struct `Counterparty`



<pre><code><b>struct</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Counterparty">Counterparty</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_MerklePrefix"></a>

## Struct `MerklePrefix`



<pre><code><b>struct</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_MerklePrefix">MerklePrefix</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new">new</a>(client_id: <a href="_String">string::String</a>, versions: <a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;, state: u64, delay_period: u64, counterparty: <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Counterparty">connection_end::Counterparty</a>): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_counterparty"></a>

## Function `new_counterparty`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_counterparty">new_counterparty</a>(client_id: <a href="_String">string::String</a>, connection_id: <a href="_String">string::String</a>, prefix: <a href="">vector</a>&lt;u8&gt;): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Counterparty">connection_end::Counterparty</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_version"></a>

## Function `new_version`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_version">new_version</a>(identifier: <a href="_String">string::String</a>, <a href="">features</a>: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_versions"></a>

## Function `new_versions`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_new_versions">new_versions</a>(identifiers: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;, <a href="">features</a>: <a href="">vector</a>&lt;<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;&gt;): <a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_delay_period"></a>

## Function `delay_period`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_delay_period">delay_period</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_versions"></a>

## Function `versions`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_versions">versions</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): &<a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_state"></a>

## Function `state`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_state">state</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_state"></a>

## Function `set_state`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_state">set_state</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<b>mut</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>, state: u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_versions"></a>

## Function `set_versions`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_versions">set_versions</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<b>mut</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>, versions: <a href="">vector</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>&gt;)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_client_id"></a>

## Function `client_id`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_client_id">client_id</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_client_id"></a>

## Function `conn_counterparty_client_id`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_client_id">conn_counterparty_client_id</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_connection_id"></a>

## Function `conn_counterparty_connection_id`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_connection_id">conn_counterparty_connection_id</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_conn_counterparty_connection_id"></a>

## Function `set_conn_counterparty_connection_id`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_conn_counterparty_connection_id">set_conn_counterparty_connection_id</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<b>mut</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>, connection_id: <a href="_String">string::String</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_key_prefix"></a>

## Function `conn_counterparty_key_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_conn_counterparty_key_prefix">conn_counterparty_key_prefix</a>(<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end">connection_end</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): &<a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_counterparty_connection_id"></a>

## Function `counterparty_connection_id`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_counterparty_connection_id">counterparty_connection_id</a>(counterparty: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Counterparty">connection_end::Counterparty</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_counterparty_client_id"></a>

## Function `counterparty_client_id`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_counterparty_client_id">counterparty_client_id</a>(counterparty: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Counterparty">connection_end::Counterparty</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_features"></a>

## Function `version_features`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_features">version_features</a>(<a href="">version</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>): &<a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_features_mut"></a>

## Function `version_features_mut`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_features_mut">version_features_mut</a>(<a href="">version</a>: &<b>mut</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>): &<b>mut</b> <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_identifier"></a>

## Function `version_identifier`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_identifier">version_identifier</a>(<a href="">version</a>: &<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>): &<a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_identifier_mut"></a>

## Function `version_identifier_mut`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_version_identifier_mut">version_identifier_mut</a>(<a href="">version</a>: &<b>mut</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>): &<b>mut</b> <a href="_String">string::String</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_version_features"></a>

## Function `set_version_features`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_version_features">set_version_features</a>(<a href="">version</a>: &<b>mut</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>, <a href="">features</a>: <a href="">vector</a>&lt;<a href="_String">string::String</a>&gt;)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_version_identifier"></a>

## Function `set_version_identifier`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_set_version_identifier">set_version_identifier</a>(<a href="">version</a>: &<b>mut</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>, identifier: <a href="_String">string::String</a>)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_default"></a>

## Function `default`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_default">default</a>(): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_default_version"></a>

## Function `default_version`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_default_version">default_version</a>(): <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_Version">connection_end::Version</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_decode_proto"></a>

## Function `decode_proto`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_decode_proto">decode_proto</a>(buf: <a href="">vector</a>&lt;u8&gt;): <a href="_Option">option::Option</a>&lt;<a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_encode_proto"></a>

## Function `encode_proto`



<pre><code><b>public</b> <b>fun</b> <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_encode_proto">encode_proto</a>(end: <a href="connection_end.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_connection_end_ConnectionEnd">connection_end::ConnectionEnd</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>
