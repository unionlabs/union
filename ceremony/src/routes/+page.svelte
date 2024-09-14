<script lang="ts">
  import {user} from "$lib/stores/user.svelte.ts"
  import H1 from "$lib/components/typography/H1.svelte"
  import {ContributorState} from "$lib/stores/state.svelte.ts"
  import Ceremony from "$lib/components/Ceremony.svelte"
  import Join from "$lib/components/Join.svelte"

  //This could be set with context API if we expand the app a lot.
  let contributor: ContributorState = new ContributorState()

  $effect(() => {
    const userId = user.session?.user.id
    if (userId) contributor.setUserId(userId)
  })
</script>

<!--Maybe add loading state to handle text jump-->
{#if contributor.loggedIn}
  {#if contributor.allowanceState === "invited"}
    <Ceremony {contributor}/>
  {:else if contributor.allowanceState === "waitingList"}
    <H1>Your on the list</H1>
  {:else}
    <Join {contributor}/>
  {/if}
{:else}
  <H1>Welcome to union ceremony</H1>
{/if}

