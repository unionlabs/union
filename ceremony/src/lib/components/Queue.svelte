<script lang="ts">
import { getNumberSuffix } from "$lib/utils/utils.js"
import Blink from "$lib/components/Blink.svelte"
import SwimLoad from "$lib/components/SwimLoad.svelte"
import Warning from "$lib/components/Warning.svelte"
import type {Contributor} from "$lib/state/contributor.svelte.ts";
import Print from "$lib/components/TerminalApp/Print.svelte";

type Props = {
  contributor: Contributor
}

let { contributor }: Props = $props()

window.addEventListener("beforeunload", (e: BeforeUnloadEvent) => {
  e.preventDefault()
  e.returnValue = ""
})
</script>

<div class="p-8 w-full max-w-4xl flex flex-col items-center">
  <Print class="mb-6">You are <span class="!text-union-accent-500">{contributor.queueState.position}<span
          class="lowercase">{getNumberSuffix(contributor.queueState.position)}</span> </span> in queue</Print>

  <SwimLoad max={contributor.queueState.count} current={contributor.queueState.position}/>
  <div class="mb-4 text-center">
    <Print>Queue length: <span class="text-union-accent-500">{contributor.queueState.count}</span></Print>
    <Print>Waiting time: <span class="text-union-accent-500">{contributor.queueState.estimatedTime} minutes</span>
      (est.).
    </Print>
  </div>
<Warning />
</div>