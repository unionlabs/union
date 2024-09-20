<script lang="ts">
import { getNumberSuffix } from "$lib/utils/utils.js"
import H2 from "$lib/components/typography/H2.svelte"
import H3 from "$lib/components/typography/H3.svelte"
import Blink from "$lib/components/Blink.svelte"
import SwimLoad from "$lib/components/SwimLoad.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import { ContributorState } from "$lib/stores/state.svelte.js"
import { start } from "$lib/client"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()
</script>

<H1 class="mb-4 text-7xl">
  <Blink/>
</H1>
<div class="p-8 w-full max-w-4xl flex flex-col items-center">
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