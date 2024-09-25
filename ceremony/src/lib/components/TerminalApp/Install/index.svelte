<script lang="ts">
  import {onMount} from "svelte"
  import {type DetectedOS, detectOS, sleep} from "$lib/utils/utils.ts"
  import Linux from "$lib/components/TerminalApp/Install/Linux.svelte";
  import MacOS from "$lib/components/TerminalApp/Install/MacOS.svelte";
  import {getState} from "$lib/state/index.svelte.ts";
  import SelectOS from "$lib/components/TerminalApp/Install/SelectOS.svelte";

  let os = $state<DetectedOS | undefined>(undefined)
  let selectedOs = $state<DetectedOS | undefined>(undefined)

  const {terminal} = getState()

  onMount(async () => {
    os = await detectOS()
  })

  let change = async () => {
    terminal.updateHistory(`Loading supported os..`, {duplicate: true})
    await sleep(500)
    selectedOs = undefined
  }

  let select = async (value: DetectedOS) => {
    terminal.updateHistory(`Loading instructions..`, {duplicate: true})
    await sleep(500)
    selectedOs = value
  }

  $effect(() => {
    console.log(selectedOs)
  })

</script>

{#if !selectedOs}
  <SelectOS {select}/>
  {:else if selectedOs === "macOS"}
  <MacOS {change}/>
  {:else if selectedOs === "Linux"}
  <Linux {change}/>
{/if}

