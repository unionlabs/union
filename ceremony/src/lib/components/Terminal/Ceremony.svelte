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
  <Reward/>

{:else if contributor.state === "contributed"}
  <Thanks/>

{:else if !contributor.downloadedSecret && contributor.clientState === "idle"}
  <Secret/>

{:else if contributor.state === "verifying"}
  {terminal.updateHistory("Verifying your contribution...", {replace: true})}

{:else if contributor.clientState === "offline" || contributor.clientState === undefined}
  <Install/>

{:else if contributor.state === "inQueue"}
  <Queue/>

{:else if contributor.state === 'contribute'}
  {terminal.updateHistory("Starting contribution...", {replace: true})}

{:else if contributor.state === "contributing"}
  {terminal.updateHistory("Contributing...", {replace: true})}

{:else}
 <Print>Not able to contribute at this time</Print>

{/if}
