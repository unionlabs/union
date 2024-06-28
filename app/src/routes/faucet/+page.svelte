<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { unionAddressRegex } from "./schema.ts"
import { Label } from "$lib/components/ui/label"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import UnoBalance from "./(components)/uno-balance.svelte"
import { Button } from "$lib/components/ui/button/index.ts"
import SpinnerSVG from "$lib/components/spinner-svg.svelte"
import WalletGate from "$lib/components/wallet-gate.svelte"
import ChainsGate from "$lib/components/chains-gate.svelte"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import ExternalFaucets from "./(components)/external-faucets.svelte"
import type { DiscriminatedUnion } from "$lib/types.ts"
import request from "graphql-request"
import { writable, type Writable } from "svelte/store"
import { URLS } from "$lib/constants/index.ts"
import { faucetUnoMutation2 } from "$lib/graphql/documents/faucet.ts"
import Truncate from "$lib/components/truncate.svelte"

type FaucetState = DiscriminatedUnion<
  "kind",
  {
    IDLE: {}
    REQUESTING_TOKEN: {}
    SUBMITTING: { captchaToken: string }
    RESULT_OK: { transactionHash: string }
    RESULT_ERR: { error: string }
  }
>

let address: string = $cosmosStore.address ?? ""

const resetInput = () => {
  address = $cosmosStore.address ?? ""
}

let faucetState: Writable<FaucetState> = writable({ kind: "IDLE" })

const fetchFromFaucet = async () => {
  if ($faucetState.kind === "IDLE" || $faucetState.kind === "REQUESTING_TOKEN") {
    faucetState.set({ kind: "REQUESTING_TOKEN" })

    if (!window?.__google_recaptcha_client) return console.error("Recaptcha not loaded")

    const captchaToken = await window.grecaptcha.execute(
      "6LdaIQIqAAAAANckEOOTQCFun1buOvgGX8J8ocow",
      {
        action: "submit"
      }
    )

    faucetState.set({ kind: "SUBMITTING", captchaToken })
  }

  if ($faucetState.kind === "SUBMITTING") {
    try {
      const result = await request(URLS.GRAPHQL, faucetUnoMutation2, {
        address,
        captchaToken: $faucetState.captchaToken
      })
      if (result.faucet2 === null) {
        faucetState.set({ kind: "RESULT_ERR", error: "Empty faucet response" })
        return
      }

      if (result.faucet2.send.startsWith("ERROR")) {
        faucetState.set({ kind: "RESULT_ERR", error: `Error from faucet: ${result.faucet2.send}` })
        return
      }

      faucetState.set({ kind: "RESULT_OK", transactionHash: result.faucet2.send })
    } catch (error) {
      // @ts-ignore
      faucetState.set({ kind: "RESULT_ERR", error: `Faucet error: ${error}` })
      return
    }
  }
}
</script>

<svelte:head>
  <title>Union | Faucet</title>
  <script src="https://www.google.com/recaptcha/api.js?render=6LdaIQIqAAAAANckEOOTQCFun1buOvgGX8J8ocow" async
          defer></script>
</svelte:head>

<main class="flex flex-col gap-6 items-center max-h-full py-6 px-3 sm:px-6 w-full">
  <Card.Root class="w-full max-w-lg">
    <Card.Header>
      <Card.Title>UNO Drip Faucet</Card.Title>
      <Card.Description>Official faucet for Union's native gas token.</Card.Description>
    </Card.Header>
    <Card.Content>
      {#if $faucetState.kind === "RESULT_OK"}
        <p>Tokens sent: <a href={`https://explorer.testnet-8.union.build/union/tx/${$faucetState.transactionHash}`}><Truncate class="underline" value={$faucetState.transactionHash} type="hash"/></a></p>
      {:else if $faucetState.kind === "RESULT_ERR"}
        <p class="mb-4">Sorry, there was an error while using the faucet. Did you make sure that the address is correct?</p>
        <Button on:click={() => faucetState.set({ kind: "IDLE"})}>Retry</Button>
        <p class="mt-4 break-words text-xs">{$faucetState.error}</p>
      {:else}
      <form
        action="?"
        method="POST"
        class="flex flex-col w-full gap-4"
        name="faucet-form"
        on:submit|preventDefault={fetchFromFaucet}
      >
          <div>
            <Label for="address">Address</Label>
            <div class="flex items-start gap-2">
              <div class="w-full">
                <div class="relative w-full mb-2">
                  <Input
                    autocapitalize="none"
                    autocomplete="off"
                    autocorrect="off"
                    bind:value={address}
                    id="address"
                    pattern={unionAddressRegex.source}
                    placeholder="union14ea6..."
                    required={true}
                    spellcheck="false"
                    name="wallet-address"
                    type="text"
                    class="disabled:opacity-100 disabled:bg-black/20"
                  />
                </div>
                <div class="flex justify-between px-1">
                  <div class="text-xs">
                    <ChainsGate let:chains>
                      <WalletGate let:userAddr let:connected let:cosmosConnected>
                        <p>
                          <span class="text-muted-foreground">Balance: </span>
                          {#if cosmosConnected}
                            <UnoBalance {chains} {userAddr} {connected} />
                          {:else}
                            Connect cosmos wallet
                          {/if}
                        </p>
                      </WalletGate>
                    </ChainsGate>
                  </div>
                  {#if address !== $cosmosStore.address }
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
            </div>
          </div>
          <div class="flex flex-row items-center gap-4">
            <Button
              type="submit"
              on:click={event => {
                event.preventDefault()
                fetchFromFaucet()
               }}
              disabled={$faucetState.kind !== "IDLE"}
              class={cn('min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50')}
            >
              Submit
              {#if $faucetState.kind !== "IDLE"}
                <span class="ml-2">
                  <SpinnerSVG className="w-4 h-4" />
                </span>
              {/if}
            </Button>
            <div class="text-[10px]">This faucet is protected by reCAPTCHA and the Google <a class="underline" href="https://policies.google.com/privacy">Privacy Policy</a> and <a class="underline" href="https://policies.google.com/terms">Terms of Service</a> apply.</div>
          </div>
        <div
          class="g-recaptcha sr-only"
          data-sitekey="6LdaIQIqAAAAANckEOOTQCFun1buOvgGX8J8ocow"
          data-callback="onSubmit"
          data-size="invisible">
          ></div>
      </form>
      {/if}
    </Card.Content>
  </Card.Root>
  <ChainsGate let:chains>
    <ExternalFaucets {chains} />
  </ChainsGate>
</main>

