<script lang="ts">
import { ContributorState, getContributorState } from "$lib/stores/state.svelte.ts"
import Spinner from "$lib/components/Spinner.svelte"
import Ceremony from "$lib/components/Ceremony.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import Join from "$lib/components/Join.svelte"
import Code from "$lib/components/Code.svelte"
import Text from "$lib/components/typography/Text.svelte"
import H4 from "$lib/components/typography/H4.svelte"

const contributor: ContributorState = getContributorState()
</script>

<section class="w-full h-svh flex flex-col justify-center items-center">
  {#if contributor.loggedIn}
    {#if !contributor.allowanceState}
      <Spinner class="text-union-accent-500 size-6"/>
    {:else if contributor.allowanceState === "hasRedeemed" || contributor.allowanceState === "inQueue"}
      <Ceremony {contributor}/>
    {:else if contributor.allowanceState === "inWaitlist"}
      <H1 class="mb-4">You're on the waitlist </H1>
      <Text class="mb-12">When the ceremony opens to the public you will have the position X in queue.</Text>
      <H4 class="mb-4">Received an invite?</H4>
      <Text class="mb-4">You can skip the waitlist and join now</Text>
      <form class="flex flex-col items-center">
        <Code {contributor} secondary={true}/>
      </form>
    {:else if contributor.allowanceState === "join"}
      <Join {contributor}/>
    {/if}
  {/if}
</section>