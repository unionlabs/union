<script lang="ts">
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import H3 from "$lib/components/typography/H3.svelte"
import H2 from "$lib/components/typography/H2.svelte"
import Install from "$lib/components/Install.svelte"
import { start } from "$lib/client"
import H4 from "$lib/components/typography/H4.svelte"
import { AddressForm, type ValidState } from "$lib/components/address"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

$effect(() => {
  console.info(`ADDRESS VALIDITY STATE: ${addressValidState}`)
  if (contributor?.state === "contribute") {
    start()
  }
})

window.addEventListener("beforeunload", (e: BeforeUnloadEvent) => {
  e.preventDefault()
  e.returnValue = ""
})

let addressValidState: ValidState = $state("PENDING")
</script>

<div class="p-8 bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center justify-center flex-col min-h-48">

  {#if contributor.state === 'inQueue'}
    <H1>Your position: <span class="text-union-accent-500">{contributor.queueState.position}</span></H1>
    <H2>Queue length: <span class="text-union-accent-500">{contributor.queueState.count}</span></H2>

    <!--Todo format time correctly if we want this, can probably be thousands of minutes?-->
    <H3>Waiting time: <span class="text-union-accent-500">{contributor.queueState.estimatedTime} minutes</span> (est.).
    </H3>

    {#if contributor.clientState === 'offline'}
      <Install/>
    {/if}

  {:else if contributor.state === 'contribute'}
    <H1>Starting contribution...</H1>
  {:else if contributor.state === 'contributing'}
    <H1>Contributing...</H1>
  {:else if contributor.state === 'verifying'}
    <H1>Verifying your contribution...</H1>
  {:else if contributor.state === 'contributed'}

    <div class="flex flex-col justify-center items-center gap-4">
      <H1>Thank you! Your contribution is completed.</H1>
      <H2>Get your nft</H2>
      <AddressForm class="" onValidation={result => (addressValidState = result)}/>
    </div>

  {:else if contributor.state === 'noClient'}
    <H1>No client. Cannot start contribution.</H1>
    <Install/>

  {:else}
    <H1>Not able to contribute at this time</H1>
  {/if}

</div>


<div class="absolute bottom-10 left-10">
  <H4>Client: <span class="text-union-accent-500">{contributor.clientState}</span></H4>
</div>