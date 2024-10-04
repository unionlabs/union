<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { onMount } from "svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import Waitlist from "$lib/components/Terminal/Waitlist.svelte"
import Join from "$lib/components/Terminal/Join.svelte"
import Authenticate from "$lib/components/Terminal/Authenticate.svelte"
import Install from "$lib/components/Terminal/Install/index.svelte"
import Thanks from "$lib/components/Terminal/Thanks.svelte"
import Reward from "$lib/components/Terminal/Reward.svelte"
import Secret from "$lib/components/Terminal/Secret.svelte"
import Missed from "$lib/components/Terminal/Missed.svelte"
import Queue from "$lib/components/Terminal/Queue.svelte"

const { user, terminal, contributor } = getState()

onMount(() => {
  terminal.setTab(1)
})
</script>

{#if user.loading}
  <Print>loading...</Print>
{:else}
  {#if user.session}

    {#if contributor.state === "contributed"}
      <Thanks/>

    {:else if contributor.state === "verifying"}
      {terminal.setStep(9)}
      {terminal.updateHistory({text: "Verifying your contribution...", replace: true})}

    {:else if contributor.state === "missed"}
      <Missed/>

    {:else if contributor.clientState === "offline"}
      <Install/>

    {:else if !contributor.storedSecret && contributor.clientState === "idle"}
      <Secret/>

    {:else if contributor.userWallet === null}
      <Reward/>

    {:else}

      {#if contributor.currentUserState === "hasRedeemed" || contributor.currentUserState === "inQueue"}
        {#if contributor.state === "inQueue"}
          <Queue/>

        {:else if contributor.state === 'contribute'}
          {terminal.setStep(9)}
          {terminal.updateHistory({text: "Starting contribution...", replace: true})}

        {:else if contributor.state === "contributing"}
          {terminal.setStep(9)}
          {terminal.updateHistory({text: "Contributing...", replace: true})}

        {:else}

          <Print>Loading</Print>
        {/if}

      {:else if contributor.currentUserState === "inWaitlist"}
        <Waitlist/>

      {:else if contributor.currentUserState === "join"}
        <Join/>

      {/if}
    {/if}
  {:else if user.session === null && terminal.tab === 1}
    <Authenticate/>
  {/if}
{/if}

