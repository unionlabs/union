<script lang="ts">
  import type {ContributorState} from "$lib/stores/state.svelte.ts"
  import H1 from "$lib/components/typography/H1.svelte"
  import H3 from "$lib/components/typography/H3.svelte"
  import H2 from "$lib/components/typography/H2.svelte"
  import {start} from "$lib/client"
  import {AddressForm, type ValidState} from "$lib/components/address"
  import Tweet from "$lib/components/Tweet.svelte"
  import SwimLoad from "$lib/components/SwimLoad.svelte"
  import {getNumberSuffix} from "$lib/utils/utils.ts"
  import Status from "$lib/components/Status.svelte"

  type Props = {
    contributor: ContributorState
  }

  let {contributor}: Props = $props()

  $effect(() => {
    if (contributor?.contributionState === "contribute" && contributor.state !== "contributing") {
      console.log("Call client start")
      start()
    }
  })

  let addressValidState: ValidState = $state("PENDING")
</script>

<div class="p-8 w-full flex items-center justify-center flex-col">

  {#if !contributor.userWallet}
    <div class="text-center">
      <H2 class="mb-2">Get your nft</H2>
      <AddressForm class="" onValidation={result => (addressValidState = result)} {contributor}/>
    </div>
  {:else}

    {#if contributor.state === 'inQueue'}

      <Status {contributor}/>

      <div class="border p-8 w-full max-w-4xl flex flex-col items-center">
        <H1 class="mb-6">You are <span class="!text-union-accent-500">{contributor.queueState.position}<span
                class="lowercase">{getNumberSuffix(contributor.queueState.position)}</span> </span> in queue</H1>

        <SwimLoad max={contributor.queueState.count} current={contributor.queueState.position}/>

        <div class="mb-4 text-center">
          <H2>Queue length: <span class="text-union-accent-500">{contributor.queueState.count}</span></H2>
          <H3>Waiting time: <span class="text-union-accent-500">{contributor.queueState.estimatedTime} minutes</span>
            (est.).
          </H3>
        </div>
      </div>

    {:else if contributor.state === 'contribute'}
      <Status {contributor}/>
      <H1>Starting contribution...</H1>

    {:else if contributor.state === 'contributing'}
      <Status {contributor}/>
      <H1>Contributing...</H1>

    {:else if contributor.state === 'verifying'}
      <Status {contributor}/>
      <H1>Verifying your contribution...</H1>

    {:else if contributor.state === 'contributed'}

      <div class="flex flex-col justify-center items-center gap-4">
        <H1>Thank you! Your contribution is completed.</H1>
        <Tweet/>
      </div>

    {:else if contributor.state === 'noClient'}

      <Status {contributor}/>
      <H1>No client. Cannot start contribution.</H1>

    {:else}
      <H1>Not able to contribute at this time</H1>

    {/if}
  {/if}

</div>


<div class="absolute bottom-10 flex flex-col px-8 text-center gap-4">
</div>