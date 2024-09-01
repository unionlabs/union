
<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient"></a>

# Module `0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::LightClient`



-  [Resource `State`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_State)
-  [Struct `Proof`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Proof)
-  [Struct `ZKP`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ZKP)
-  [Struct `Timestamp`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Timestamp)
-  [Struct `LightHeader`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_LightHeader)
-  [Struct `Header`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Header)
-  [Struct `ClientState`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ClientState)
-  [Struct `MerkleRoot`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_MerkleRoot)
-  [Struct `ConsensusState`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ConsensusState)
-  [Constants](#@Constants_0)
-  [Function `create_client`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_create_client)
-  [Function `latest_height`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_latest_height)
-  [Function `update_client`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_update_client)
-  [Function `verify_membership`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_verify_membership)
-  [Function `verify_non_membership`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_verify_non_membership)
-  [Function `new_client_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_client_state)
-  [Function `new_consensus_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_consensus_state)
-  [Function `new_merkle_root`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_merkle_root)
-  [Function `get_timestamp_at_height`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_timestamp_at_height)
-  [Function `get_client_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_client_state)
-  [Function `get_consensus_state`](#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_consensus_state)


<pre><code><b>use</b> <a href="">0x1::any</a>;
<b>use</b> <a href="">0x1::aptos_hash</a>;
<b>use</b> <a href="">0x1::bcs</a>;
<b>use</b> <a href="">0x1::bn254_algebra</a>;
<b>use</b> <a href="">0x1::crypto_algebra</a>;
<b>use</b> <a href="">0x1::from_bcs</a>;
<b>use</b> <a href="">0x1::hash</a>;
<b>use</b> <a href="">0x1::object</a>;
<b>use</b> <a href="">0x1::option</a>;
<b>use</b> <a href="">0x1::smart_table</a>;
<b>use</b> <a href="">0x1::string</a>;
<b>use</b> <a href="">0x1::timestamp</a>;
<b>use</b> <a href="">0x1::vector</a>;
<b>use</b> <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408::height</a>;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_State"></a>

## Resource `State`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_State">State</a> <b>has</b> store, key
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Proof"></a>

## Struct `Proof`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Proof">Proof</a> <b>has</b> drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ZKP"></a>

## Struct `ZKP`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ZKP">ZKP</a> <b>has</b> drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Timestamp"></a>

## Struct `Timestamp`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Timestamp">Timestamp</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_LightHeader"></a>

## Struct `LightHeader`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_LightHeader">LightHeader</a> <b>has</b> drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Header"></a>

## Struct `Header`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_Header">Header</a> <b>has</b> drop
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ClientState"></a>

## Struct `ClientState`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ClientState">ClientState</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_MerkleRoot"></a>

## Struct `MerkleRoot`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_MerkleRoot">MerkleRoot</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ConsensusState"></a>

## Struct `ConsensusState`



<pre><code><b>struct</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ConsensusState">ConsensusState</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ALPHA_G1"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ALPHA_G1">ALPHA_G1</a>: <a href="">vector</a>&lt;u8&gt; = [153, 168, 24, 193, 103, 1, 111, 127, 109, 2, 216, 64, 5, 165, 237, 31, 124, 108, 25, 196, 221, 241, 87, 51, 182, 122, 204, 1, 41, 7, 103, 9, 255, 129, 13, 157, 51, 116, 128, 128, 105, 193, 234, 30, 93, 38, 58, 144, 207, 129, 129, 185, 139, 65, 88, 5, 121, 113, 118, 53, 122, 206, 199, 8];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_BETA_G2"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_BETA_G2">BETA_G2</a>: <a href="">vector</a>&lt;u8&gt; = [116, 40, 132, 234, 24, 160, 14, 243, 24, 116, 213, 252, 85, 17, 177, 143, 169, 57, 29, 198, 155, 151, 27, 137, 138, 45, 191, 198, 68, 3, 63, 21, 101, 109, 201, 47, 31, 148, 220, 23, 0, 38, 205, 128, 33, 46, 81, 96, 210, 83, 158, 126, 139, 64, 136, 93, 29, 96, 183, 112, 210, 95, 53, 153];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_DELTA_G2"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_DELTA_G2">DELTA_G2</a>: <a href="">vector</a>&lt;u8&gt; = [235, 4, 77, 219, 149, 30, 155, 40, 237, 167, 218, 147, 171, 163, 65, 239, 44, 150, 164, 214, 24, 44, 167, 133, 163, 32, 24, 201, 200, 3, 212, 5, 252, 185, 240, 74, 49, 201, 136, 162, 245, 166, 71, 16, 255, 175, 225, 1, 131, 29, 97, 71, 37, 155, 84, 228, 93, 71, 224, 209, 24, 76, 94, 41];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_GAMMA_ABC_G1"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_GAMMA_ABC_G1">GAMMA_ABC_G1</a>: <a href="">vector</a>&lt;<a href="">vector</a>&lt;u8&gt;&gt; = [ByteArray([129, 146, 83, 48, 148, 29, 83, 216, 206, 193, 196, 66, 16, 246, 200, 130, 254, 232, 44, 74, 233, 124, 182, 75, 79, 134, 67, 39, 229, 67, 24, 39, 6, 36, 203, 115, 37, 168, 159, 234, 122, 210, 203, 222, 71, 138, 123, 163, 142, 202, 24, 187, 161, 240, 36, 246, 114, 177, 248, 156, 198, 66, 51, 37]), ByteArray([202, 75, 18, 93, 94, 26, 46, 192, 226, 38, 114, 67, 79, 190, 156, 160, 227, 202, 21, 176, 194, 14, 22, 233, 2, 14, 214, 244, 113, 190, 13, 11, 12, 224, 112, 182, 168, 185, 95, 104, 112, 20, 216, 61, 224, 159, 158, 254, 51, 202, 175, 22, 170, 146, 229, 236, 136, 131, 118, 211, 235, 154, 11, 19]), ByteArray([199, 144, 196, 161, 145, 138, 177, 46, 126, 60, 54, 0, 91, 47, 92, 188, 245, 64, 140, 237, 152, 3, 53, 113, 118, 12, 124, 244, 213, 147, 158, 2, 217, 241, 238, 106, 156, 19, 182, 235, 190, 46, 17, 218, 178, 63, 86, 0, 4, 15, 203, 131, 59, 181, 121, 143, 174, 207, 157, 69, 16, 5, 241, 44])];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_GAMMA_G2"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_GAMMA_G2">GAMMA_G2</a>: <a href="">vector</a>&lt;u8&gt; = [25, 182, 113, 158, 66, 196, 46, 209, 223, 70, 250, 8, 200, 112, 197, 36, 26, 82, 145, 59, 101, 217, 180, 54, 121, 224, 137, 194, 224, 187, 22, 34, 207, 58, 72, 156, 167, 146, 127, 79, 129, 64, 10, 46, 189, 115, 154, 147, 91, 206, 179, 34, 66, 100, 239, 248, 226, 72, 49, 26, 233, 107, 231, 160];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_HMAC_I"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_HMAC_I">HMAC_I</a>: <a href="">vector</a>&lt;u8&gt; = [117, 89, 91, 83, 66, 116, 122, 101, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54, 54];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_HMAC_O"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_HMAC_O">HMAC_O</a>: <a href="">vector</a>&lt;u8&gt; = [31, 51, 49, 57, 40, 30, 16, 15, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92, 92];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_PEDERSEN_G"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_PEDERSEN_G">PEDERSEN_G</a>: <a href="">vector</a>&lt;u8&gt; = [90, 229, 109, 192, 20, 168, 19, 119, 18, 244, 88, 70, 88, 186, 111, 126, 57, 12, 195, 152, 146, 249, 126, 86, 202, 133, 152, 135, 216, 216, 240, 19, 135, 25, 189, 159, 250, 43, 186, 150, 57, 81, 218, 46, 8, 186, 146, 255, 193, 4, 155, 162, 241, 253, 125, 127, 3, 176, 44, 19, 248, 246, 125, 37];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_PEDERSEN_G_ROOT_SIGMA_NEG"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_PEDERSEN_G_ROOT_SIGMA_NEG">PEDERSEN_G_ROOT_SIGMA_NEG</a>: <a href="">vector</a>&lt;u8&gt; = [175, 91, 78, 48, 18, 58, 52, 67, 57, 50, 29, 214, 33, 181, 253, 249, 205, 152, 112, 98, 89, 40, 250, 7, 35, 95, 1, 28, 223, 4, 161, 2, 104, 99, 202, 226, 242, 176, 192, 206, 69, 126, 129, 173, 37, 160, 104, 251, 28, 184, 96, 38, 9, 107, 232, 227, 247, 92, 85, 167, 65, 225, 191, 175];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_PRIME_R_MINUS_ONE"></a>



<pre><code><b>const</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_PRIME_R_MINUS_ONE">PRIME_R_MINUS_ONE</a>: <a href="">vector</a>&lt;u8&gt; = [0, 0, 0, 240, 147, 245, 225, 67, 145, 112, 185, 121, 72, 232, 51, 40, 93, 88, 129, 129, 182, 69, 80, 184, 41, 160, 49, 225, 114, 78, 100, 48];
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_create_client"></a>

## Function `create_client`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_create_client">create_client</a>(ibc_signer: &<a href="">signer</a>, client_id: <a href="_String">string::String</a>, client_state: <a href="_Any">any::Any</a>, consensus_state: <a href="_Any">any::Any</a>): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_latest_height"></a>

## Function `latest_height`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_latest_height">latest_height</a>(client_id: <a href="_String">string::String</a>): <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_update_client"></a>

## Function `update_client`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_update_client">update_client</a>(client_id: <a href="_String">string::String</a>, client_msg: <a href="_Any">any::Any</a>): (<a href="">vector</a>&lt;<a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>&gt;, u64)
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_verify_membership"></a>

## Function `verify_membership`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_verify_membership">verify_membership</a>(_client_id: <a href="_String">string::String</a>, _height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, _proof: <a href="">vector</a>&lt;u8&gt;, _prefix: <a href="">vector</a>&lt;u8&gt;, _path: <a href="">vector</a>&lt;u8&gt;, _value: <a href="">vector</a>&lt;u8&gt;): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_verify_non_membership"></a>

## Function `verify_non_membership`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_verify_non_membership">verify_non_membership</a>(_client_id: <a href="_String">string::String</a>, _height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, _proof: <a href="">vector</a>&lt;u8&gt;, _prefix: <a href="">vector</a>&lt;u8&gt;, _path: <a href="">vector</a>&lt;u8&gt;): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_client_state"></a>

## Function `new_client_state`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_client_state">new_client_state</a>(<a href="">chain_id</a>: <a href="_String">string::String</a>, trusting_period: u64, unbonding_period: u64, max_clock_drift: u64, frozen_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>, latest_height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ClientState">LightClient::ClientState</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_consensus_state"></a>

## Function `new_consensus_state`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_consensus_state">new_consensus_state</a>(<a href="">timestamp</a>: u64, app_hash: <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_MerkleRoot">LightClient::MerkleRoot</a>, next_validators_hash: <a href="">vector</a>&lt;u8&gt;): <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_ConsensusState">LightClient::ConsensusState</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_merkle_root"></a>

## Function `new_merkle_root`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_new_merkle_root">new_merkle_root</a>(<a href="">hash</a>: <a href="">vector</a>&lt;u8&gt;): <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_MerkleRoot">LightClient::MerkleRoot</a>
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_timestamp_at_height"></a>

## Function `get_timestamp_at_height`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_timestamp_at_height">get_timestamp_at_height</a>(_client_id: <a href="_String">string::String</a>, _height: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): u64
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_client_state"></a>

## Function `get_client_state`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_client_state">get_client_state</a>(client_id: <a href="_String">string::String</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>



<a id="0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_consensus_state"></a>

## Function `get_consensus_state`



<pre><code><b>public</b> <b>fun</b> <a href="cometbls_light_client.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_LightClient_get_consensus_state">get_consensus_state</a>(client_id: <a href="_String">string::String</a>, <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height">height</a>: <a href="height.md#0x9e1ceeb126ea73b3c29a6cfd1151e06a948cbc4ce06c81fc38119edc0edb5408_height_Height">height::Height</a>): <a href="">vector</a>&lt;u8&gt;
</code></pre>
