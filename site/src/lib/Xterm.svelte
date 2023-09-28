<script lang="ts">
	import { onMount } from 'svelte';
	import { ApolloClient, InMemoryCache, gql } from '@apollo/client/core';
	import type { Terminal } from 'xterm';
	import type { ApolloQueryResult } from '@apollo/client';
	import BlogLayout from '../mdsvex/BlogLayout.svelte';
	import ButtonA from './ButtonA.svelte';

	const client = new ApolloClient({
	  uri: 'https://graphql.union.build/v1/graphql',
	  cache: new InMemoryCache(),
	});

	const FETCH_EVENT = gql`
		query FetchEvent($id: Int!) {
		  demo_txes_by_pk(id: $id) {
		    data
		    created_at
		  }
		}
	`

	const FETCH_LATEST_ID = gql`
		query GetLatestId {
		  demo_txes(limit: 1, order_by: {id: desc}) {
		    id
		  }
		}
	`

	let terminal: null | Terminal;
	let terminalElement: HTMLElement;
	let latestId: null | number = null;
	const replayOffset = 300;

	let logLines: {network: String, action: String, logLine: String}[] = [];


	let scrollAnchor: HTMLElement;
	let scroller: HTMLElement;



    const filter = (r: ApolloQueryResult<any>): null | { network: String, action: String, logLine: String } => {
			let data = r.data.demo_txes_by_pk;
			if (data === null) {
				return null
			}
			data = r.data.demo_txes_by_pk.data;

			let network;
			let action;

			if ('EthereumMinimal' in data) {
				network = "union"
                data = data['EthereumMinimal']
			}

			if ('CometblsMinimal' in data) {
				network = "sepolia"
                data = data['CometblsMinimal']
			}

			if ('Fetch' in data) {
                action = "fetching "
				data = data["Fetch"]["data"]
			}

			if ('Event' in data) {
                action = "observed event "
				data = data["Event"]["data"]
			}

			if ('Msg' in data) {
                action = "sending message "
				data = data["Msg"]["data"]
			}

            if (network === undefined) {
				return null
			}

			return { network, action, logLine: JSON.stringify(data)}
	}

	const sleep = (ms: number) =>  new Promise(r => setTimeout(r, ms));

	const worker = async (latestIdWorker) => {
		const startHeight = latestIdWorker - replayOffset
		let i = startHeight;
    while (true) {
			i++;
      await sleep(2000);
      const response = await client.query({
       query: FETCH_EVENT,
       variables: { id: i },
      });

			const newLine = filter(response);
			if (newLine != null) {
				logLines = [newLine, ...logLines];
			}

			if (i === (latestIdWorker - 1)) {
				i = startHeight;
			} 
    }
  }
	
	onMount(async () => {
		let response = await client.query({ query: FETCH_LATEST_ID, });
		latestId = response.data.demo_txes[0].id; 		
    worker(latestId);
	})
</script>


<div class="h-[332px] md:h-[432px] my-8">
	<div style="margin: 0 auto;" class="absolute max-w-4xl p-4 md:shadow-2xl left-0 md:left-[16px] right-0 md:right-[16px] bg-black text-xs sm:text-sm font-jetbrains md:rounded-xl">
			<div style="flex-direction: column-reverse;" bind:this={terminalElement} class="overflow-scroll flex scrollbar-hide h-[300px] md:h-[400px]" >
			{#each logLines as {network, action, logLine}}
				<div class="p-0"><span class={ network == "union" ? "text-accent" : "text-yellow-300"}>[{network}] </span><span>{action}</span><span class="text-gray-400">{logLine}</span></div>
			{/each}
			</div>
	</div>
</div>

<style>
	/* For Webkit-based browsers (Chrome, Safari and Opera) */
	.scrollbar-hide::-webkit-scrollbar {
	    display: none;
	}

	/* For IE, Edge and Firefox */
	.scrollbar-hide {
	    -ms-overflow-style: none;  /* IE and Edge */
	    scrollbar-width: none;  /* Firefox */
	}
</style>

