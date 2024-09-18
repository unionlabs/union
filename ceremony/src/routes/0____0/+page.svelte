<script lang="ts">
import { ContributorState, getContributorState } from "$lib/stores/state.svelte.ts"
import Spinner from "$lib/components/Spinner.svelte"
import Ceremony from "$lib/components/Ceremony.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import Join from "$lib/components/Join.svelte"

const contributor: ContributorState = getContributorState()
</script>

{#if contributor.loggedIn}
  {#if !contributor.allowanceState}
    <Spinner class="text-union-accent-500 size-6"/>
  {:else if contributor.allowanceState === "hasRedeemed" || contributor.allowanceState === "inQueue"}
    <Ceremony {contributor}/>
  {:else if contributor.allowanceState === "inWaitlist"}
    <H1>You're on the list</H1>
  {:else if contributor.allowanceState === "join"}
    <Join {contributor}/>
  {/if}
{/if}