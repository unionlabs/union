<script lang="ts">
  import Print from "$lib/components/TerminalApp/Print.svelte";
  import {onMount} from "svelte";
  import {getState} from "$lib/state/index.svelte.ts";
  import LoadingBar from "$lib/components/SwimLoad.svelte";

  const {contributor, terminal} = getState()

  window.addEventListener("beforeunload", (e: BeforeUnloadEvent) => {
    e.preventDefault()
    e.returnValue = ""
  })

  onMount(() => {
    terminal.updateHistory("You are in queue")
  })
</script>

<Print>------</Print>
<Print>Your position:  {contributor.queueState.position ?? 33}</Print>
<Print>Queue length: {contributor.queueState.count ?? 347}</Print>
<Print>Estimated waiting time: {contributor.queueState.estimatedTime ?? 990} minutes</Print>
<LoadingBar max={contributor.queueState.count ?? 347} current={contributor.queueState.position ?? 33}/>
<Print>Your MPC Client is connected.</Print>
<Print>Do not close this tab or your Terminal.</Print>
<Print>Ensure you have a reliable internet connection and that your computer does not go to sleep.</Print>

