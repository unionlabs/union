<script lang="ts">
	import { Tendermint37Client as TmClient, type NewBlockEvent, type TxEvent } from '@cosmjs/tendermint-rpc';
	import { ethers } from 'ethers';

	const cosmosUrl = 'localhost:26657';
	const evmUrl = 'ws://localhost:8546';

	let cosmosHeight = 0;
	let evmHeight = 0;
	let cosmosContract = 'union1hzzmyldthamamhxgsgvl3utudjngkmyn055udp9c9yzccfncxj7s2lhq9d';
	let evmContract = '0x433488cec14C4478e5ff18DDC7E7384Fc416f148';
	let listening = false;
	let txs: any[] = [];

	const runCosmos = async () => {
		const client = await TmClient.connect(cosmosUrl);
		client.subscribeNewBlock().addListener({
			next: (event: NewBlockEvent) => {
				cosmosHeight = event.header.height;
			},
			error: (err: any) => {},
			complete: () => {}
		});
	};

	const runEvm = async () => {
		const provider = new ethers.WebSocketProvider(evmUrl);
		provider.on('block', (height) => {
			evmHeight = height;
		});
	};

	runCosmos();
	runEvm();

	const start = async () => {
		listening = true;
		const provider = new ethers.WebSocketProvider(evmUrl);
		const abi = ['event Ring(bool)'];
		// IbcHandler.Capabilities["ping-pong"][0]
		// const pingPongAddress = await provider.getStorage(
		//     evmContract,
		//     ethers.keccak256(ethers.keccak256(
		//         ethers.solidityPacked(["string", "uint256"], ["ping-pong", 10])
		//     ))
		// );
		const contract = new ethers.Contract(evmContract, abi, provider);
		contract.on('Ring', (ping, event) => {
			const action = ping ? 'ping' : 'pong';
			txs = txs.concat([
				{
					network: 'evm',
					height: event.log.blockNumber,
					hash: ethers.hexlify(event.log.transactionHash),
					action: action
				}
			]);
		});
		const client = await TmClient.connect(cosmosUrl);
		client.subscribeTx().addListener({
			next: (event: TxEvent) => {
				const wasmEvent = event.result.events.find((e) => e.type === 'wasm');
				if (wasmEvent) {
					const contractAddress = wasmEvent.attributes.find((a) => a.key == '_contract_address');
					if (contractAddress) {
						if (contractAddress.value === cosmosContract) {
							const action = wasmEvent.attributes.find((a) => a.key == 'action');
							if (action) {
								txs = txs.concat([
									{
										network: 'cosmos',
										height: event.height,
										hash: ethers.hexlify(event.hash),
										action: action.value
									}
								]);
							}
						}
					}
				}
			},
			error: (err: any) => {},
			complete: () => {}
		});
	};
</script>

<section>
	<article>
		<h1>PingPong Protocol</h1>
		<p>Latest Cosmos height: {cosmosHeight}</p>
		<p>Latest EVM height: {evmHeight}</p>
		<form>
			<fieldset>
				<label for="cosmos_contract">Cosmos contract:</label>
				<input
					id="cosmos_contract"
					type="text"
					size="71"
					bind:value={cosmosContract}
					disabled={listening}
				/>
				<label for="evm_contract">EVM contract:</label>
				<input
					id="evm_contract"
					type="text"
					size="46"
					bind:value={evmContract}
					disabled={listening}
				/>
				<br />
				{#if listening}
					<button disabled>Check out the transactions below</button>
				{:else}
					<button type="submit" on:click={start}>Start listening</button>
				{/if}
			</fieldset>
		</form>
		{#if listening}
			<table>
				<tr>
					<th>Time</th>
					<th>Network</th>
					<th>Height</th>
					<th>Tx</th>
					<th>Action</th>
				</tr>
				<tbody>
					{#each txs as tx}
						<tr>
							<td>{new Date().toLocaleTimeString()}</td>
							<td>{tx.network}</td>
							<td>{tx.height}</td>
							<td>{tx.hash.substring(0, 10)}...</td>
							<td>{tx.action}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</article>
</section>

