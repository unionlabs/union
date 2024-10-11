<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { sleep } from "$lib/utils/utils.ts"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"
import { COMMAND } from "$lib/constants"

type Props = {
  change: () => void
}

const { terminal } = getState()
let { change }: Props = $props()

let showButtons = $state(true)

onMount(() => {
  terminal.setStep(3)
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_macos" }])
  const messages = [
    {
      text: "You must have OrbStack installed in order to contribute, because Docker Desktop is too slow. If you use Docker Desktop it is extremely likely that you will lose your contribution slot."
    },
    { text: "---" },
    {
      text: '1. <a class="underline-offset-4 decoration-union-accent-500 underline" href="https://orbstack.dev/" target="_blank">Install OrbStack</a>'
    },
    { text: "2. Open OrbStack from the Applications/ folder" },
    { text: "3. Click allow on the OrbStack popups" },
    {
      text: "4. Open Terminal from the Applications/Utilities/ folder"
    },
    {
      text: "5. Paste the following command in Terminal to start the MPC client:"
    },
    { text: "---", duplicate: true },
    { text: COMMAND },
    { text: "---", duplicate: true },
    {
      text: "Once the MPC client is running you can return to this page."
    },
    { text: "---", duplicate: true },
    {
      text: "If the MPC client is running but you still see this page, ensure that you are using Chrome, Firefox, or Brave. Also, make sure to temporarily turn off ad-blockers or browser shields (especially in Brave).",
      type: "warning"
    }
  ]

  messages.forEach(msg => {
    terminal.updateHistory(msg)
  })
})

const copy = async () => {
  showButtons = false
  terminal.updateHistory({ text: "Copying command...", duplicate: true })
  await sleep(500)
  await navigator.clipboard.writeText(COMMAND)
  terminal.updateHistory({ text: "Command copied!", duplicate: true })
  await sleep(500)
  showButtons = true
}

const selectDifferentOs = () => {
  showButtons = false
  change()
}

function trigger(value: "copy" | "select") {
  if (value === "copy") {
    copy()
  } else if (value === "select") {
    selectDifferentOs()
  }
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if showButtons}
  <Buttons
          data={[{text: "Copy command", action: "copy"}, {text: "Select different OS", action: "select"}]}
          trigger={(value) => trigger(value)}/>

{/if}
