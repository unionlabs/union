<script lang="ts">
	import { onMount } from 'svelte';
	import { ApolloClient, InMemoryCache, ApolloProvider, gql, type ApolloQueryResult } from '@apollo/client';
	import type { Terminal } from 'xterm';
    import { json } from '@sveltejs/kit';

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

			return network + action + JSON.stringify(data) + '\r\n'
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

								if (terminal == null) {
									console.error("Terminal has not been initiated correctly prior to starting worker");
									return;
								}
                console.log(result)
				let line = filter(result);
				if (line !== null) {
					terminal.write(line)
				}
              })
            .catch(err => {
							if (terminal == null) {
								console.error("Terminal has not been initiated correctly prior to starting worker");
								return;
							}
              console.error(err)
              terminal.write(err)
            });
        }
    }
	
	onMount(async () => {
		const xterm = (await import("xterm"))	
		terminal = new xterm.Terminal()
		terminal.open(terminalElement);
    worker()
	})
</script>

<div bind:this={terminalElement} />



<style>
	.observer {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
	}
	
	
	/* Xterm.css */
	:global(.xterm) {
			cursor: text;
			position: relative;
			user-select: none;
			-ms-user-select: none;
			-webkit-user-select: none;
	}

	:global(.xterm.focus),
	:global(.xterm:focus) {
			outline: none;
	}

	:global(.xterm .xterm-helpers) {
			position: absolute;
			top: 0;
			/**
			 * The z-index of the helpers must be higher than the canvases in order for
			 * IMEs to appear on top.
			 */
			z-index: 5;
	}

	:global(.xterm .xterm-helper-textarea) {
			padding: 0;
			border: 0;
			margin: 0;
			/* Move textarea out of the screen to the far left, so that the cursor is not visible */
			position: absolute;
			opacity: 0;
			left: -9999em;
			top: 0;
			width: 0;
			height: 0;
			z-index: -5;
			/** Prevent wrapping so the IME appears against the textarea at the correct position */
			white-space: nowrap;
			overflow: hidden;
			resize: none;
	}

	:global(.xterm .composition-view) {
			/* TODO: Composition position got messed up somewhere */
			background: #000;
			color: #FFF;
			display: none;
			position: absolute;
			white-space: nowrap;
			z-index: 1;
	}

	:global(.xterm .composition-view.active) {
			display: block;
	}

	:global(.xterm .xterm-viewport) {
			/* On OS X this is required in order for the scroll bar to appear fully opaque */
			background-color: #000;
			overflow-y: scroll;
			cursor: default;
			position: absolute;
			right: 0;
			left: 0;
			top: 0;
			bottom: 0;
	}

	:global(.xterm .xterm-screen) {
			position: relative;
	}

	:global(.xterm .xterm-screen canvas) {
			position: absolute;
			left: 0;
			top: 0;
	}

	:global(.xterm .xterm-scroll-area) {
			visibility: hidden;
	}

	:global(.xterm-char-measure-element) {
			display: inline-block;
			visibility: hidden;
			position: absolute;
			top: 0;
			left: -9999em;
			line-height: normal;
	}

	:global(.xterm.enable-mouse-events) {
			/* When mouse events are enabled (eg. tmux), revert to the standard pointer cursor */
			cursor: default;
	}

	:global(.xterm.xterm-cursor-pointer),
	:global(.xterm .xterm-cursor-pointer) {
			cursor: pointer;
	}

	:global(.xterm.column-select.focus) {
			/* Column selection mode */
			cursor: crosshair;
	}

	:global(.xterm .xterm-accessibility),
	:global(.xterm .xterm-message) {
			position: absolute;
			left: 0;
			top: 0;
			bottom: 0;
			right: 0;
			z-index: 10;
			color: transparent;
	}

	:global(.xterm .live-region) {
			position: absolute;
			left: -9999px;
			width: 1px;
			height: 1px;
			overflow: hidden;
	}

	:global(.xterm-dim) {
			opacity: 0.5;
	}

	:global(.xterm-underline) {
			text-decoration: underline;
	}

	:global(.xterm-strikethrough) {
			text-decoration: line-through;
	}

	:global(.xterm-screen .xterm-decoration-container .xterm-decoration) {
		z-index: 6;
		position: absolute;
	}

	:global(.xterm-decoration-overview-ruler) {
			z-index: 7;
			position: absolute;
			top: 0;
			right: 0;
			pointer-events: none;
	}

	:global(.xterm-decoration-top) {
			z-index: 2;
			position: relative;
	}

</style>
