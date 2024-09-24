<script lang="ts">
import Code from "$lib/components/Code.svelte"
import Text from "$lib/components/typography/Text.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import { getNumberSuffix } from "$lib/utils/utils.ts"
import Button from "$lib/components/Button.svelte"
import { fade } from "svelte/transition"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

let haveCode = $state(false)
</script>

<div class="flex flex-col items-center text-center gap-8">
  {#if !haveCode}
    <div class="flex flex-col items-center gap-4" in:fade>
      <H1>You're on the waitlist </H1>
      <Text>When the ceremony opens to the public, you will be <span
              class="text-union-accent-500">{contributor.waitListPosition}{getNumberSuffix(contributor.waitListPosition)}</span>
        in the public queue.
        <br>You will receive an email 12-18 hours before the public phase begins.
      </Text>
      <Button variant="secondary" onclick={() => haveCode = !haveCode}>I have a code</Button>
    </div>
  {:else}
    <div class="flex flex-col items-center gap-4" in:fade>
      <H1>Plunge into the abyss</H1>
      <Text>Ditch the surface crowds and dive straight into shark whale territory.</Text>
      <form class="flex flex-col items-center">
        <Code {contributor}/>
      </form>
      <button class="text-white/50 mt-6 hover:text-white transition-colors" onclick={() => haveCode = !haveCode}>&lt-
        Back
      </button>
    </div>
  {/if}
</div>