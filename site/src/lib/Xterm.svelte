<script lang="ts">
	import { onMount } from 'svelte';
	import { ApolloClient, InMemoryCache, gql } from '@apollo/client/core';
	import type { Terminal } from 'xterm';

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

	let logLines: String[] = [];


	const scrollToBottom = node => {
    const scroll = () => node.scroll({
        top: node.scrollHeight,
        behavior: 'smooth',
    });
    scroll();

    return { update: scroll }
};


    const filter = (r: ApolloQueryResult<any>): null | string  => {
			let data = r.data.demo_txes_by_pk;
			if (data === null) {
				return null
			}
			data = r.data.demo_txes_by_pk.data;

			let network;
			let action;

			if ('EthereumMinimal' in data) {
				network = "[union]: "
                data = data['EthereumMinimal']
			}

			if ('CometblsMinimal' in data) {
				network = "[sepolia]: "
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

			return network + action + JSON.stringify(data)
	}



	const worker = async () => {
	    for (let i = 0; i < 100000000; i++) {
	        await new Promise(r => setTimeout(r, 2000));
	        client.query({
             query: FETCH_EVENT,
             variables: {
                id: i
             },
            }).then((result) => {

                console.log(result)
								const newLine = filter(result);
								if (newLine != null) {
									logLines = [...logLines, newLine];
									scrollToBottom(terminalElement);
								}

	        })
        }
    }
	
	onMount(async () => {
    worker()
	})
</script>


<div class="relative h-80">
	<div bind:this={terminalElement} class="overflow-auto absolute left-0 right-0 bg-black h-80 text-sm font-jetbrains">
		{#each logLines as logline}
			<div>{logline}</div>
		{/each}
	</div>
</div>

