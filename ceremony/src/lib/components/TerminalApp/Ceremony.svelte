<script lang="ts">
  import {generateSecret, start} from "$lib/client"
  import Reward from "$lib/components/Reward.svelte"
  import Download from "$lib/components/Download.svelte"
  import Queue from "$lib/components/Queue.svelte"
  import Install from "$lib/components/TerminalApp/Install/index.svelte"
  import Thanks from "$lib/components/Thanks.svelte"
  import Warning from "$lib/components/Warning.svelte"
  import {getState} from "$lib/state/index.svelte.ts";
  import Print from "$lib/components/TerminalApp/Print.svelte";

  const { contributor, client , terminal, user} = getState()

  async function generate() {
    await generateSecret(user.session?.user.email)
  }

  $effect(() => {
    console.log(contributor.userWallet)
    if (contributor?.contributionState === "contribute" && contributor.state !== "contributing") {
      start()
    }

    if (client.state !== "offline") {
      generate()
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

{:else if contributor.state !== 'contributed'}
  <Thanks />

{:else if contributor.state === 'verifying'}
  {terminal.updateHistory("Verifying your contribution...")}

{:else if contributor.state === "inQueue"}
  <Queue {contributor}/>

{:else if contributor.state === 'contribute'}
  {terminal.updateHistory("Starting contribution...")}

{:else if contributor.state === 'contributing'}
  {terminal.updateHistory("Contributing...")}

{:else if !contributor.downloadedSecret && client.state === "idle"}
  <Download />

{:else if client.state === 'noClient'}
  <Install />

{:else}
  <Print>Loading...</Print>

{/if}
