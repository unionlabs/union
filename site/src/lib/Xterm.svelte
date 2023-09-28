<script lang="ts">
	import { onMount } from 'svelte';
	import { ApolloClient, InMemoryCache, gql } from '@apollo/client/core';
	import type { Terminal } from 'xterm';
	import type { ApolloQueryResult } from '@apollo/client';

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

	let terminal: null | Terminal;
	let terminalElement: HTMLElement;

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

	const worker = async () => {
	    for (let i = 0; i < 100000000; i++) {
	        await new Promise(r => setTimeout(r, 200));
	        client.query({
             query: FETCH_EVENT,
             variables: {
                id: i
             },
            }).then(async (result) => {

                console.log(result)
								const newLine = filter(result);
								if (newLine != null) {
									logLines = [...logLines, newLine];
								}

	        })
        }
    }
	
	onMount(async () => {
		terminalElement.scroll(0,1);
    worker();
	})
</script>


<div class="relative h-80 my-4">
	<div bind:this={terminalElement} class="absolute p-2 overflow-scroll left-0 right-0 bg-black h-80 text-sm font-jetbrains">
			<div class="terminal-line h-[100.1%]"/>
			{#each logLines as {network, action, logLine}}
				<div class="terminal-line p-0"><span class={ network == "union" ? "text-accent" : "text-yellow-300"}>[{network}] </span><span>{action}</span><span class="text-gray-400">{logLine}</span></div>
			{/each}
		  <div bind:this={scrollAnchor} id="anchor"/>
	</div>
</div>


<style>
	.terminal-line {
		overflow-anchor: none;
	}
	#anchor {
		overflow-anchor: auto;
		height: 1px;
	}
</style>

