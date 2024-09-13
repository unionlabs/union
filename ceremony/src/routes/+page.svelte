<script lang="ts">
  import {user} from "$lib/stores/user.svelte.ts"
  import H1 from "$lib/components/typography/H1.svelte";
  import {ContributorState} from "$lib/stores/state.svelte.ts";
  import H4 from "$lib/components/typography/H4.svelte";
  import Ceremony from "$lib/components/Ceremony.svelte";
  import {AddressForm, type ValidState} from "$lib/components/address/index.ts"

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
  <H4>Client: {contributor.clientState}</H4>
</div>


<style>
    .deep-sea-rise {
        opacity: 0;
        transform: scale(0.5) translateY(140px);
        filter: brightness(0.1);
        animation: riseFromDepth 2.5s ease-out forwards;
    }

    .deep-sea-rise.visible {
        opacity: 1;
    }

    @keyframes riseFromDepth {
        0% {
            opacity: 0;
            transform: scale(0.5) translateY(140px);
            filter: brightness(0.1);
        }

        100% {
            opacity: 1;
            transform: scale(1) translateY(0);
            filter: brightness(1);
        }
    }

    .fade-in-text {
        opacity: 0;
        animation: fadeInText 1.5s ease-out forwards;
        animation-delay: 2s;
    }

    @keyframes fadeInText {
        0% {
            opacity: 0;
        }
        100% {
            opacity: 1;
        }
    }
</style>
