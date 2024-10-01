<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import Reward from "$lib/components/Terminal/Reward.svelte"
import Thanks from "$lib/components/Terminal/Thanks.svelte"
import Queue from "$lib/components/Terminal/Queue.svelte"
import Install from "$lib/components/Terminal/Install/index.svelte"
import Secret from "$lib/components/Terminal/Secret.svelte"
import { onDestroy } from "svelte"

const { contributor, terminal } = getState()

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if !contributor.userWallet}
  {terminal.setStep(4)}
  <Reward/>

{:else if contributor.state === "contributed"}
  {terminal.setStep(10)}
  <Thanks/>

{:else if !contributor.downloadedSecret && contributor.clientState === "idle"}
  {terminal.setStep(6)}
  <Secret/>

{:else if contributor.state === "verifying"}
  {terminal.setStep(9)}
  {terminal.updateHistory({text: "Verifying your contribution...", replace: true})}

{:else if contributor.clientState === "offline" || contributor.clientState === undefined}
  {terminal.setStep(5)}
  <Install/>

{:else if contributor.state === "inQueue"}
  {terminal.setStep(7)}
  <Queue/>

{:else if contributor.state === 'contribute'}
  {terminal.setStep(8)}
  {terminal.updateHistory({text: "Starting contribution...", replace: true})}

{:else if contributor.state === "contributing"}
  {terminal.setStep(9)}
  {terminal.updateHistory({text: "Contributing...", replace: true})}

{:else if contributor.contributionState === "missed"}
  {terminal.setStep(9)}
  {terminal.updateHistory({text: "Too bad, you missed your slot.", replace: true})}

{:else}
  {terminal.updateHistory({text: "Loading", replace: true})}

{/if}
