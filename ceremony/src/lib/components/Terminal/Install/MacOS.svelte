<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { sleep } from "$lib/utils/utils.ts"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"

type Props = {
  change: () => void
}

const { terminal } = getState()
let { change }: Props = $props()

let showButtons = $state(true)

let command =
  "mkdir -p ceremony && docker pull ghcr.io/unionlabs/union/mpc-client:latest && docker run -v $(pwd)/ceremony:/ceremony -w /ceremony -p 4919:4919 --rm -it ghcr.io/unionlabs/union/mpc-client:latest"

onMount(() => {
  const messages = [
    { text: "---" },
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
    { text: command },
    { text: "---", duplicate: true },
    {
      text: "Once the MPC client is running you can return to this page."
    },
    {
      text: "If the MPC client is running but you still see this page, ensure that you are using either Chrome, FireFox or Brave. For Brave, disable the shields in the address bar."
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
  await navigator.clipboard.writeText(command)
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
