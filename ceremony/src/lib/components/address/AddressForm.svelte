<script lang="ts">
import clsx from "clsx"
import { toast } from "svelte-sonner"
import { watch, Debounced } from "runed"
import type { ValidState } from "./index.ts"
import { isValidBech32Address } from "./validator.ts"
import type { HTMLInputAttributes } from "svelte/elements"
import { insertWalletData } from "$lib/supabase"
import { user } from "$lib/stores/user.svelte.ts"

interface Props extends HTMLInputAttributes {
  class?: string
  onValidation: (valid: ValidState) => ValidState
}

let { onValidation, class: className = "", ...props }: Props = $props()

let inputText = $state("")
const debouncedInputText = new Debounced(
  () => inputText,
  /**
   * TODO: change this to 1s and during debounce, show a loading state
   */
  0
)

let validState: ValidState = $state("PENDING")

$effect(() => {
  if (validState === "INVALID") toast.error(`Address is not valid`)
})

const onAddressSubmit = async (event: Event) => {
  event.preventDefault()
  if (!debouncedInputText.current) return
  const addressValidation = isValidBech32Address(debouncedInputText.current)
  validState = addressValidation ? "VALID" : "INVALID"
  onValidation(validState)

  const userId = user.session?.user.id
  if (validState === "VALID") {
    try {
      if (!userId) return
      const result = await insertWalletData({
        id: userId,
        wallet: debouncedInputText.current
      })
      if (result) {
        toast.success("Wallet address saved successfully")
      } else {
        toast.error("Failed to save wallet address")
      }
    } catch (error) {
      console.error("Error saving wallet address:", error)
      toast.error("An error occurred while saving the wallet address")
    }
  }
}
</script>

<form class="flex flex-col gap-2 min-w-[355px]">
  <input
    {...props}
    type="text"
    autocorrect="off"
    autocomplete="off"
    spellcheck="false"
    autocapitalize="none"
    bind:value={inputText}
    placeholder="union1qp0wtsfltjk9rnvyu3fkdv0s0skp4y5y3py96f"
    class={clsx([
      className,
      'text-md font-supermolot h-9 px-2 outline-none border-2',
      validState === 'VALID'
        ? 'border-2 border-green-500'
        : validState === 'INVALID'
          ? 'border-2 border-red-500'
          : 'border-transparent',
    ])}
  />
  <button
    type="button"
    onclick={onAddressSubmit}
    disabled={inputText.length === 0}
    class={clsx([
      'hover:font-bold hover:bg-[#5FDFFC]',
      'uppercase text-black w-full bg-[#A0ECFD] text-md font-supermolot h-9 px-2 font-semibold',
    ])}
  >
    submit
  </button>
</form>

<style lang="postcss"></style>
