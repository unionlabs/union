<script lang="ts">
import { start } from "$lib/client"
import { getState } from "$lib/state/index.svelte.ts"
import Reward from "$lib/components/Terminal/Reward.svelte"
import Thanks from "$lib/components/Terminal/Thanks.svelte"
import Queue from "$lib/components/Terminal/Queue.svelte"
import Install from "$lib/components/Terminal/Install/index.svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import Secret from "$lib/components/Terminal/Secret.svelte"

const { contributor, client, terminal } = getState()

$effect(() => {
  if (contributor?.contributionState === "contribute" && contributor.state !== "contributing") {
    start()
  }

  if (contributor.state === "contributing" || contributor.state === "inQueue") {
    window.addEventListener("beforeunload", (e: BeforeUnloadEvent) => {
      e.preventDefault()
      e.returnValue = ""
    })
  }
})
</script>

{#if !contributor.userWallet}
  <Reward />

{:else if contributor.contributionState === 'contributed'}
  <Thanks />

{:else if contributor.contributionState === 'verifying'}
  {terminal.updateHistory("Verifying your contribution...")}

{:else if contributor.contributionState === 'contribute'}
  {terminal.updateHistory("Starting contribution...")}
  <Print>Not starting?</Print>
  <button>&gt Start</button>

{:else if contributor.contributionState === "contribute"}
  {terminal.updateHistory("Contributing...")}

{:else if contributor.state === "inQueue"}
  <Queue />

{:else if !contributor.downloadedSecret && client.state === "idle"}
  <Secret />

{:else if client.state === 'noClient'}
  <Install />

{:else}
  <Print>Loading...</Print>

{/if}
