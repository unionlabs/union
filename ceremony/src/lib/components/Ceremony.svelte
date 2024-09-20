<script lang="ts">
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import { generateSecret, start } from "$lib/client"
import Reward from "$lib/components/Reward.svelte"
import Download from "$lib/components/Download.svelte"
import Queue from "$lib/components/Queue.svelte"
import Install from "$lib/components/Install.svelte"
import Thanks from "$lib/components/Thanks.svelte"
import { user } from "$lib/stores/user.svelte.ts"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

async function generate() {
  const email = user.session?.user.email
  await generateSecret(email)
}

$effect(() => {
  if (contributor?.contributionState === "contribute" && contributor.state !== "contributing") {
    start()
  }

  if (contributor.clientState !== "offline") {
    generate()
  }
})

window.addEventListener("beforeunload", (e: BeforeUnloadEvent) => {
  e.preventDefault()
  e.returnValue = ""
})
</script>

<div class="p-8 w-full flex items-center justify-center flex-col">

  {#if !contributor.userWallet}
    <Reward {contributor}/>
  {:else if contributor.state === 'contributed'}
    <Thanks {contributor}/>
  {:else if contributor.clientState === 'offline'}
    <Install {contributor}/>
  {:else if !contributor.downloadedSecret}
    <Download {contributor}/>
  {:else if contributor.state === "inQueue"}
    <Queue {contributor}/>
  {:else if contributor.state === 'contribute'}
    <H1>Starting contribution...</H1>
  {:else if contributor.state === 'contributing'}
    <H1>Contributing...</H1>
  {:else if contributor.state === 'verifying'}
    <H1>Verifying your contribution...</H1>
  {:else}
    <H1>Not able to contribute at this time</H1>
  {/if}

</div>

<div class="absolute bottom-10 flex flex-col px-8 text-center gap-4"></div>