<script lang="ts">
import { toast } from "svelte-sonner"
import { unionAddressRegex } from "./schema.ts"
import { Label } from "$lib/components/ui/label"
import { debounce } from "$lib/utilities/index.ts"
import LockLockedIcon from "virtual:icons/lucide/lock"
import { createMutation } from "@tanstack/svelte-query"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import LockOpenIcon from "virtual:icons/lucide/lock-open"
import UnoBalance from "./(components)/uno-balance.svelte"
import { Button } from "$lib/components/ui/button/index.ts"
import SpinnerSVG from "$lib/components/spinner-svg.svelte"
import WalletGate from "$lib/components/wallet-gate.svelte"
import { getUnoFromFaucet } from "$lib/mutations/faucet.ts"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import ExternalFaucets from "./(components)/external-faucets.svelte"
import { isValidCosmosAddress } from "$/lib/wallet/utilities/validate.ts"

let userInput = false
let address: string = $cosmosStore.address ?? ""

$: if (!userInput && $cosmosStore.address !== address) {
  address = $cosmosStore.address ?? ""
}

const handleInput = (event: Event) => {
  address = (event.target as HTMLInputElement).value
  userInput = true
}

const resetInput = () => {
  userInput = false
  address = $cosmosStore.address ?? ""
}

const debounceDelay = 3_500
let submissionStatus: "idle" | "submitting" | "submitted" | "error" = "idle"
let inputState: "locked" | "unlocked" = $cosmosStore.address ? "locked" : "unlocked"
const onLockClick = () => (inputState = inputState === "locked" ? "unlocked" : "locked")

let opacity = 0

let focused = false
let input: HTMLInputElement
let position = { x: 0, y: 0 }

const handleFocus = () => ([focused, opacity] = [true, 1])
const handleBlur = () => ([focused, opacity] = [false, 0])
const handleMouseEnter = () => (opacity = 1)
const handleMouseLeave = () => (opacity = 0)

function handleMouseMove(event: MouseEvent) {
  if (!input || focused) return
  const rect = input.getBoundingClientRect()
  position = { x: event.clientX - rect.left, y: event.clientY - rect.top }
}

const mutation = createMutation({
  mutationKey: ["faucetRequest"],
  mutationFn: async () => getUnoFromFaucet(address),
  onError: error => {
    console.error("Error during the faucet request:", error)
    submissionStatus = "error"
    toast.error("Faucet request failed.")
  },
  onSuccess: data => {
    toast.success("Faucet request successful!")
    console.log("Faucet request successful:", data)
  }
})

const debouncedSubmit = debounce(() => {
  if (!isValidCosmosAddress(address)) {
    toast.error("Invalid address")
    return
  }
  $mutation.mutate()
  submissionStatus = "submitted"
  toast.error("Faucet request submitted!")
}, debounceDelay)

const handleSubmit = () => {
  submissionStatus = "submitting"
  toast.loading("Submitting faucet request..")
  debouncedSubmit()
}
</script>

<svelte:head>
  <title>Union | Faucet</title>
</svelte:head>

  <main class="flex flex-col gap-6 items-center max-h-full py-6 px-3 sm:px-6 w-full">
    <Card.Root class="w-full max-w-lg">
      <Card.Header>
        <Card.Title>UNO Faucet</Card.Title>
      </Card.Header>
      <Card.Content>
        <form class="space-y-8" on:submit|preventDefault={handleSubmit}>
          <div class="relative flex flex-col gap-4">
            <div class="grid w-full items-center gap-2 mb-4">
              <Label for="address">Address</Label>
              <div class="flex items-start gap-2">
                <div class="w-full">
                  <div class="relative w-full mb-2">
                    <Input
                      autocapitalize="none"
                      autocomplete="off"
                      autocorrect="off"
                      bind:value={address}
                      disabled={inputState === 'locked'}
                      id="address"
                      on:blur={handleBlur}
                      on:focus={handleFocus}
                      on:input={handleInput}
                      on:mouseenter={handleMouseEnter}
                      on:mouseleave={handleMouseLeave}
                      on:mousemove={handleMouseMove}
                      pattern={unionAddressRegex.source}
                      placeholder="union14ea6..."
                      required={true}
                      spellcheck="false"
                      type="text"
                      class="disabled:opacity-100 disabled:bg-black/20"
                    />
                  </div>
                  <div class="flex justify-between px-1">
                    <div class="text-xs">
                      <ChainsGate let:chains>
                        <WalletGate>
                          <div slot="connected" let:userAddr>
                            <p>
                              <span class="text-muted-foreground">Balance: </span>
                              <UnoBalance {chains} {userAddr} />
                            </p>
                          </div>
                          <div slot="disconnected">
                            <p>Please check wallet connections</p>
                          </div>
                        </WalletGate>
                      </ChainsGate>
                    </div>
                    {#if userInput}
                      <button
                        type="button"
                        on:click={resetInput}
                        class="text-xs text-muted-foreground hover:text-primary transition"
                      >
                        Reset
                      </button>
                    {/if}
                  </div>
                </div>
                <Button
                  aria-label="Toggle address lock"
                  class="px-3"
                  on:click={onLockClick}
                  variant="ghost"
                >
                  {#if inputState === 'locked'}
                    <LockLockedIcon class="size-4.5" />
                  {:else}
                    <LockOpenIcon class="size-4.5" />
                  {/if}
                </Button>
              </div>
            </div>
            <div class="flex flex-col gap-4 sm:flex-row">
              <Button class="w-full sm:w-fit" type="submit">
                Submit
                {#if submissionStatus === 'submitting'}
                  <span class="ml-2">
                    <SpinnerSVG className="w-4 h-4" />
                  </span>
                {/if}
              </Button>
            </div>
          </div>
        </form>
      </Card.Content>
    </Card.Root>
    <ChainsGate let:chains>
      <ExternalFaucets {chains}/>
    </ChainsGate>
  </main>
