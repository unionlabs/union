<script lang="ts">
import {
  snapAddress,
  getSnapAddress,
  ensureSnapInstalled,
  ensureSnapConnected,
  ensureSnapChainInitialized
} from "$/lib/snap.ts"
import { onMount } from "svelte"
import { balanceQuery } from "$/lib/queries/balance"
import Header from "$lib/components/Header.svelte"

let error: any

let pollingIntervalMS = 2500

onMount(async () => {
  await ensureSnapInstalled()
  await ensureSnapConnected()
  await getSnapAddress()
  await ensureSnapChainInitialized()
})

$: unoUnionBalance = balanceQuery({ address: $snapAddress, chain: "UNION", asset: "UNO" })
</script>

<Header />

<main class="mt-12 flex min-h-full min-w-full flex-col items-center justify-center space-y-6">
 <p></p>
</main>
