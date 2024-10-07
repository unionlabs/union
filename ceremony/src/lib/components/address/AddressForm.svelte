<script lang="ts">
import type { ValidState } from "./index.ts"
import { isValidBech32Address } from "./validator.ts"
import type { HTMLInputAttributes } from "svelte/elements"
import { insertWalletData } from "$lib/supabase"
import { user } from "$lib/state/session.svelte.ts"
import { getState } from "$lib/state/index.svelte.ts"
import { sleep } from "$lib/utils/utils.ts"
import { axiom } from "$lib/utils/axiom.ts"
import { onDestroy } from "svelte"

interface Props extends HTMLInputAttributes {
  class?: string
  validation: (state: ValidState) => void
}

let { validation, class: className = "", ...props }: Props = $props()

const { contributor, terminal } = getState()

let inputText = $state("")

let validState: ValidState = $state(undefined)

const onAddressSubmit = async (event: Event) => {
  event.preventDefault()

  if (!inputText) return

  if (inputText === "skip" || inputText === "Skip") {
    skip()
    return
  }

  validation("PENDING")
  terminal.updateHistory({ text: "Checking address...", duplicate: true })
  await sleep(1000)
  const addressValidation = isValidBech32Address(inputText)
  validState = addressValidation ? "VALID" : "INVALID"
  validation(validState)

  const userId = user.session?.user.id

  if (validState === "VALID") {
    try {
      if (!userId) return
      const result = await insertWalletData({
        id: userId,
        wallet: inputText
      })
      if (result) {
        terminal.updateHistory({ text: "Saving address...", duplicate: true })
        await sleep(2000)
        terminal.updateHistory({ text: "Wallet address saved successfully", duplicate: true })
        axiom.ingest("monitor", [{ user: user.session?.user.id, type: "added_address" }])
        await sleep(2000)
        contributor.checkUserWallet(user.session?.user.id)
        validation("SAVED")
      } else {
        terminal.updateHistory({ text: "Failed to save wallet address", duplicate: true })
      }
    } catch (error) {
      console.error("Error saving wallet address:", error)
      terminal.updateHistory({
        text: "An error occurred while saving the wallet address",
        duplicate: true
      })
    }
  } else if (validState === "INVALID") {
    terminal.updateHistory({ text: "Wallet address not valid, try again.", duplicate: true })
  }
}

const skip = async () => {
  terminal.updateHistory({ text: "Skipping reward step", duplicate: true })
  validation("SKIPPED")
  try {
    if (!contributor.userId) return
    const result = await insertWalletData({
      id: contributor.userId,
      wallet: "SKIPPED"
    })
    if (result) {
      terminal.updateHistory({ text: "Saving to db...", duplicate: true })
      axiom.ingest("monitor", [{ user: user.session?.user.id, type: "skipped_address" }])
      await sleep(2000)
      contributor.userWallet = "SKIPPED"
      validation("SKIPPED")
    } else {
      terminal.updateHistory({ text: "Failed to save wallet address", duplicate: true })
    }
  } catch (error) {
    console.error("Error saving wallet address:", error)
    terminal.updateHistory({
      text: "An error occurred while saving the wallet address",
      duplicate: true
    })
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    event.preventDefault()
    onAddressSubmit(event)
  }
}
</script>

<form class="w-full">
  <input
          autofocus
          {...props}
          type="text"
          autocorrect="off"
          autocomplete="off"
          spellcheck="false"
          autocapitalize="none"
          bind:value={inputText}
          onkeydown={handleKeyDown}
          class="inline-flex bg-transparent w-full text-union-accent-500 outline-none focus:ring-0 focus:border-none"
  />
</form>

<style lang="postcss"></style>
