<script lang="ts">
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import { start } from "$lib/client"
import Text from "$lib/components/typography/Text.svelte"
import Reward from "$lib/components/Reward.svelte"
import Download from "$lib/components/Download.svelte"
import Queue from "$lib/components/Queue.svelte"
import Install from "$lib/components/Install.svelte"
import Thanks from "$lib/components/Thanks.svelte"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

$effect(() => {
  if (contributor?.contributionState === "contribute" && contributor.state !== "contributing") {
    console.log("Call client start")
    start()
  }
})

window.addEventListener("beforeunload", (e: BeforeUnloadEvent) => {
  e.preventDefault()
  e.returnValue = ""
})
</script>

<div class="p-8 w-full flex items-center justify-center flex-col">

  {#if !contributor.userWallet}
    <Reward {contributor} />
  {:else if contributor.state === 'inQueue'}
    {#if contributor.clientState === "offline"}
      <Install {contributor} />
    {:else if !contributor.downloadedSecret}
      <Download {contributor} />
    {:else}
      <Queue {contributor}/>
    {/if}
  {:else if contributor.state === 'contribute'}
    <H1>Starting contribution...</H1>
  {:else if contributor.state === 'contributing'}
    <H1>Contributing...</H1>
  {:else if contributor.state === 'verifying'}
    <H1>Verifying your contribution...</H1>
  {:else if contributor.state === 'contributed'}
    <Thanks {contributor}/>
  {:else}
    <H1>Not able to contribute at this time</H1>
  {/if}
  {#if contributor.state !== "contributed"}
    <div class="text-center font-bold text-lg">
      <Text>You are connected to your MPC Client.</Text>
      <Text>Do not close this tab or your terminal running the MPC Client.</Text>
    </div>
  {/if}
</div>

<div class="absolute bottom-10 flex flex-col px-8 text-center gap-4"></div>