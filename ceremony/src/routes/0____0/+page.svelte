<script lang="ts">
import { ContributorState, getContributorState } from "$lib/stores/state.svelte.ts"
import Spinner from "$lib/components/Spinner.svelte"
import Ceremony from "$lib/components/Ceremony.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import Join from "$lib/components/Join.svelte"
import Code from "$lib/components/Code.svelte"

const contributor: ContributorState = getContributorState()
</script>

{#if contributor.loggedIn}
  {#if !contributor.allowanceState}
    <Spinner class="text-union-accent-500 size-6"/>
  {:else if contributor.allowanceState === "hasRedeemed" || contributor.allowanceState === "inQueue"}
    <Ceremony {contributor}/>
  {:else if contributor.allowanceState === "inWaitlist"}
    <H1 class="mb-4">You're on the waitlist</H1>
    <form class="flex flex-col items-center">
      <Code {contributor} />
    </form>
  {:else if contributor.allowanceState === "join"}
    <Join {contributor}/>
  {/if}
{/if}