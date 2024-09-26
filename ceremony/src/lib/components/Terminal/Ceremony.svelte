<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import Reward from "$lib/components/Terminal/Reward.svelte"
import Thanks from "$lib/components/Terminal/Thanks.svelte"
import Queue from "$lib/components/Terminal/Queue.svelte"
import Install from "$lib/components/Terminal/Install/index.svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import Secret from "$lib/components/Terminal/Secret.svelte"
import { onDestroy } from "svelte"

const { contributor, terminal } = getState()

onDestroy(() => {
  terminal.clearHistory()
})

</script>

{#if !contributor.userWallet}
  {terminal.setStep(4)}
  <Reward />

{:else if contributor.state === "contributed"}
  {terminal.setStep(10)}
  <Thanks/>

{:else if !contributor.downloadedSecret && contributor.clientState === "idle"}
  {terminal.setStep(6)}
  <Secret/>

{:else if contributor.state === "verifying"}
  {terminal.setStep(9)}
  {terminal.updateHistory("Verifying your contribution...", {replace: true})}

{:else if contributor.clientState === "offline" || contributor.clientState === undefined}
  {terminal.setStep(5)}
  <Install/>

{:else if contributor.state === "inQueue"}
  {terminal.setStep(7)}
  <Queue />

{:else if contributor.state === 'contribute'}
  {terminal.setStep(8)}
  {terminal.updateHistory("Starting contribution...", {replace: true})}

{:else if contributor.state === "contributing"}
  {terminal.setStep(9)}
  {terminal.updateHistory("Contributing...", {replace: true})}

{:else}
 <Print>Not able to contribute at this time</Print>
{/if}
