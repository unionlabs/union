<script lang="ts">
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import H3 from "$lib/components/typography/H3.svelte"
import H2 from "$lib/components/typography/H2.svelte"
import { start } from "$lib/client"
import { AddressForm, type ValidState } from "$lib/components/address"
import Tweet from "$lib/components/Tweet.svelte"
import SwimLoad from "$lib/components/SwimLoad.svelte"
import { getNumberSuffix } from "$lib/utils/utils.ts"
import Status from "$lib/components/Status.svelte"
import Text from "$lib/components/typography/Text.svelte"
import Button from "$lib/components/Button.svelte"
import H4 from "$lib/components/typography/H4.svelte"
import Blink from "$lib/components/Blink.svelte"

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

let addressValidState: ValidState = $state("PENDING")

window.addEventListener("beforeunload", (e: BeforeUnloadEvent) => {
  e.preventDefault()
  e.returnValue = ""
})
</script>

<div class="p-8 w-full flex items-center justify-center flex-col">

  {#if !contributor.userWallet}

    <div class="text-center flex flex-col items-center gap-4">
      <H2 class="">Add an address</H2>
      <Text class="">You may receive rewards for successful contributions.</Text>
      <AddressForm class="" onValidation={result => (addressValidState = result)} {contributor}/>
      <Text class="py-8">Or</Text>
      <H4>I don't want rewards</H4>
      <Text>You can contribute without adding an address</Text>
      <Button class="bg-transparent text-white hover:text-white border-2 border-white hover:bg-neutral-800">Skip
        rewards
      </Button>
    </div>

  {:else}

    {#if contributor.state === 'inQueue'}
      {#if contributor.clientState === "offline"}
        <Status {contributor}/>
      {:else}

        <H1 class="mb-4 text-7xl">
          <Blink/>
        </H1>
        <div class="p-8 w-full max-w-4xl flex flex-col items-center">
          <H1 class="mb-6">You are <span class="!text-union-accent-500">{contributor.queueState.position}<span
                  class="lowercase">{getNumberSuffix(contributor.queueState.position)}</span> </span> in queue</H1>

          <SwimLoad max={100} current={90}/>
          <div class="mb-4 text-center">
            <H2>Queue length: <span class="text-union-accent-500">{contributor.queueState.count}</span></H2>
            <H3>Waiting time: <span class="text-union-accent-500">{contributor.queueState.estimatedTime} minutes</span>
              (est.).
            </H3>
          </div>
        </div>

      {/if}
    {:else if contributor.state === 'contribute'}
      <H1>Starting contribution...</H1>

    {:else if contributor.state === 'contributing'}
      <H1>Contributing...</H1>

    {:else if contributor.state === 'verifying'}
      <H1>Verifying your contribution...</H1>

    {:else if contributor.state === 'contributed'}

      <div class="flex flex-col justify-center items-center gap-4">
        <H1>Thank you!</H1>
        <Text>Your contribution is completed. Thank you for securing the Union proving system!</Text>
        <Text>You can tweet the cryptographic attestation of your contribution for extra transparency.</Text>

        <Tweet/>
        <a href="/contributions" class="underline underline-offset-4 decoration-union-accent-500 text-white uppercase"
        >View contributions</a>

      </div>

    {:else}
      <H1>Not able to contribute at this time</H1>

    {/if}

    {#if contributor.state !== "contributed"}
      <div class="text-center font-bold text-lg">
        <Text>You are connected to your MPC Client.</Text>
        <Text>Do not close this tab or your terminal running the MPC Client.</Text>
      </div>
    {/if}
  {/if}

</div>


<div class="absolute bottom-10 flex flex-col px-8 text-center gap-4">
</div>