<script lang="ts">
	import { onMount } from 'svelte';
	import DemoButton from "$lib/DemoButton.svelte";
	import TerminalContainer from "$lib/TerminalContainer.svelte";
	import PulseSpinner from "$lib/PulseSpinner.svelte";
	import { getUnoFromFaucet } from "$lib/transferDemo";
	import { gql } from "@apollo/client/core";
	import { get, writable } from "svelte/store";
	import type { Writable } from "svelte/store";
	import { connectedToUnion, unionAccount, apolloClient } from "$lib/stores/wallets";
	import { unionUnoBalance } from "$lib/stores/wallets"; 
	import AddressesAndBalances from "./AddressesAndBalances.svelte";
	import { toFixedPoint } from "$lib/format";

	let loading: Writable<boolean> = writable(false);
	let fetchedFromFaucet: Writable<boolean> = writable(false);
	let graphqlChecked = false;

	const INSERT_DEMO_ADDRESS = gql`
		mutation InsertDemoAddress($address: String!, $identifiers: jsonb = "") {
			 insert_demo_faucet_claims_one(object: {address: $address, identifiers: $identifiers}) {
			    address
		  }
		}
	`

	let fingerprint = null

	const clickHandler = async () => {
		if (graphqlChecked && getBrowserFingerprint != null) {
				const fp = getBrowserFingerprint.default();
				const apollo = get(apolloClient);
				let uAccount = get(unionAccount);
				if (uAccount === null || apollo === null) {
					return;
				}		

				await apollo.mutate({
	       mutation: INSERT_DEMO_ADDRESS,
	       variables: { address: uAccount.address,  identifiers: { fingerprint: fp } },
	      });
				
			}

		loading.set(true);
		await getUnoFromFaucet();
		const currentBalance = get(unionUnoBalance);
		if (currentBalance === null) {
			return;
		}
		unionUnoBalance.subscribe((newBalance) => {
			if (newBalance !== null && BigInt(newBalance.amount) > BigInt(currentBalance.amount)) {
				loading.set(false);
				fetchedFromFaucet.set(true);
			}
			});
	};

	onMount(async () => {
		getBrowserFingerprint = await import ("get-browser-fingerprint");
	})
</script>


<TerminalContainer>
	{#if !$connectedToUnion || $unionUnoBalance == null}
		Complete the previous steps to continue
	{:else}
		{#if $loading}

			<div class="flex gap-4 h-[48px] items-center">
				<div>Requesting UNO from faucet</div>
				<PulseSpinner/>
			</div>
		{:else if $fetchedFromFaucet}
			<div class="flex gap-4 h-[48px] items-center">
				<div>✅ Received UNO from faucet, new balance is <span class="text-accent">{toFixedPoint(BigInt($unionUnoBalance.amount), 6)}</span></div> 
			</div>
		{:else}
				<DemoButton on:click={clickHandler}>Get UNO from faucet</DemoButton>
				<div class="flex items-center gap-2 h-[48px]">
				 <input bind:checked={graphqlChecked} type="checkbox" id="graphql_cb" class="
				    relative peer shrink-0
				    appearance-none w-4 h-4 border-2 border-cyan-400 rounded-sm bg-black
				    mt-1
				    checked:bg-cyan-400 checked:border-0
						"/>
			  <label for="graphql_cb">This is the checkbox label</label>
			  <svg
			    class="
			      absolute 
			      w-4 h-4 mt-1
			      hidden peer-checked:block
				    pointer-events-none"
			    xmlns="http://www.w3.org/2000/svg"
			    viewBox="0 0 24 24"
			    fill="none"
			    stroke="currentColor"
			    stroke-width="4"
			    stroke-linecap="round"
			    stroke-linejoin="round"
			  >
		    <polyline points="20 6 9 17 4 12"></polyline>
  </svg>
				</div>
		{/if}
	{/if}
</TerminalContainer>
