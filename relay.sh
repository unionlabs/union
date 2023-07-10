RUST_LOG=tendermint=off,beacon_api=debug,relayer=debug,info \
	cargo run -p relayer -- \
	relay-packets \
	--ibc-handler-address 0xC3d6E476103dC6F908F6d5E0baC799524b46A350 --cometbls-client-address 0x93001E081a3f48677F8bE81996260B73Be3E7741 --ics20-transfer-address 0x1516B595237ef89f69fd2e6a3BcF0EbF5834F117 --ics20-bank-address 0xE63d3C2A800b6127AaDc4d77c1ECABAC80f13a33 \
	--code-id 6dc27a4c2b3c20cc169f2273196c8b47d23501d1fd93d1a2010e30ec17f2a864 \
	--wallet 4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77 \
	--eth-beacon-rpc-api "http://localhost:9596" \
	--eth-rpc-api "ws://0.0.0.0:8546" \
	--cometbls-client-id="cometbls-new-1" --ethereum-client-id="08-wasm-2" \
	--cometbls-port-id transfer \
	--ethereum-port-id wasm.union1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrswvfu3r
	# --cometbls="cometbls-new-2/connection-2" --ethereum="08-wasm-0/connection-0" \
	# --open-channel \

# union1yyca08xqdgvjz0psg56z67ejh9xms6l436u8y58m82npdqqhmmtqrgaene

# 2023-07-10T03:09:08.075406Z  INFO relayer: channel opened cometbls_connection_info.connection_id="connection-2" cometbls_connection_info.client_id="cometbls-new-2" cometbls_channel_id="channel-0" ethereum_connection_info.connection_id="connection-0" ethereum_connection_info.client_id="08-wasm-0" ethereum_channel_id="channel-0"

#       --cometbls-client-id <COMETBLS_CLIENT_ID>
#       --ethereum-client-id <ETHEREUM_CLIENT_ID>
#       --cometbls-port-id <COMETBLS_PORT_ID>
#       --ethereum-port-id <ETHEREUM_PORT_ID>
# 2023-07-10T06:13:10.026886Z  INFO relayer: channel opened cometbls_connection_info.connection_id="connection-1" cometbls_connection_info.client_id="cometbls-new-1" cometbls_channel_id="channel-0" ethereum_connection_info.connection_id="connection-2" ethereum_connection_info.client_id="08-wasm-2" ethereum_channel_id="channel-1"
