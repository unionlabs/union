<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import H1 from "$lib/components/typography/H1.svelte"
import { ContributorState } from "$lib/stores/state.svelte.ts"
import H4 from "$lib/components/typography/H4.svelte"
import Ceremony from "$lib/components/Ceremony.svelte"
import { AddressForm, type ValidState } from "$lib/components/address/index.ts"

let addressValidState: ValidState = $state("PENDING")
let contributor: ContributorState = new ContributorState()

$effect(() => {
  console.info(`ADDRESS VALIDITY STATE: ${addressValidState}`)

  const userId = user.session?.user.id
  if (userId) contributor.setUserId(userId)
})

</script>


{#if contributor}
  {#if contributor.loggedIn}
    <Ceremony {contributor}/>
  {:else}
    <AddressForm class="" onValidation={result => (addressValidState = result)}/>
    <H1>Welcome to union ceremony</H1>
  {/if}
{/if}

<div class="absolute bottom-10 left-10">
  <H4>Client: <span class="text-union-accent-500">{contributor.clientState}</span></H4>
</div>