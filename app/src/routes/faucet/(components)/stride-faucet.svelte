<script lang="ts">
import { URLS } from "$lib/constants"
import request from "graphql-request"
import { cn } from "$lib/utilities/shadcn.ts"
import { Label } from "$lib/components/ui/label"
import { createQuery } from "@tanstack/svelte-query"
import Truncate from "$lib/components/truncate.svelte"
import * as Card from "$lib/components/ui/card/index.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import SpinnerSVG from "$lib/components/spinner-svg.svelte"
import { cosmosStore } from "$/lib/wallet/cosmos/config.ts"
import { derived, writable, type Writable } from "svelte/store"
import { strideFaucetMutation } from "$lib/graphql/queries/faucet"
import { getCosmosChainBalances } from "$lib/queries/balance/cosmos"
import { createCosmosSdkAddressRegex } from "$lib/utilities/address.ts"
import { bech32ToBech32Address, isValidBech32Address } from "@unionlabs/client"
import type { AwaitedReturnType, DiscriminatedUnion } from "$lib/utilities/types.ts"

type DydxFaucetState = DiscriminatedUnion<
  "kind",
  {
    IDLE: {}
    REQUESTING_TOKEN: {}
    SUBMITTING: { captchaToken: string }
    RESULT_OK: { message: string }
    RESULT_ERR: { error: string }
  }
>

let strideAddress = derived(cosmosStore, $cosmosStore =>
  $cosmosStore.address
    ? bech32ToBech32Address({
        address: $cosmosStore.address,
        toPrefix: "stride"
      })
    : ""
)

let strideFaucetState: Writable<DydxFaucetState> = writable({ kind: "IDLE" })

const requestStrdFromFaucet = async () => {
  if ($strideFaucetState.kind === "IDLE" || $strideFaucetState.kind === "REQUESTING_TOKEN") {
    strideFaucetState.set({ kind: "REQUESTING_TOKEN" })

    if (!window?.__google_recaptcha_client) {
      console.error("Recaptcha client not loaded")
      strideFaucetState.set({
        kind: "RESULT_ERR",
        error: "Recaptcha client not loaded"
      })
      return
    }

    if (
      typeof window.grecaptcha === "undefined" ||
      typeof window.grecaptcha.execute !== "function"
    ) {
      console.error("Recaptcha execute function not available")
      strideFaucetState.set({
        kind: "RESULT_ERR",
        error: "Recaptcha execute function not available"
      })
      return
    }

    const captchaToken = await window.grecaptcha.execute(
      "6LdaIQIqAAAAANckEOOTQCFun1buOvgGX8J8ocow",
      { action: "submit" }
    )

    strideFaucetState.set({ kind: "SUBMITTING", captchaToken })
  }

  if ($strideFaucetState.kind === "SUBMITTING") {
    try {
      const result = await request(URLS.GRAPHQL, strideFaucetMutation, {
        address: $strideAddress,
        captchaToken: $strideFaucetState.captchaToken
      })

      if (!result.stride_faucet) {
        strideFaucetState.set({
          kind: "RESULT_ERR",
          error: "Empty faucet response"
        })
        return
      }

      if (result.stride_faucet.send.startsWith("ERROR")) {
        console.error(result.stride_faucet.send)
        strideFaucetState.set({
          kind: "RESULT_ERR",
          error: result.stride_faucet.send.endsWith("ratelimited")
            ? "You already got STRD from the faucet today. Try again in 24 hours."
            : "Error from faucet"
        })
        return
      }

      strideFaucetState.set({
        kind: "RESULT_OK",
        message: result.stride_faucet.send
      })
    } catch (error) {
      console.error(error)
      strideFaucetState.set({
        kind: "RESULT_ERR",
        error: `Faucet error: ${error}`
      })
    }
  }
}

let strideBalance = createQuery(
  derived(strideAddress, $strideAddress => ({
    queryKey: ["stride-balance", $strideAddress],
    enabled: $strideAddress?.indexOf("stride") === 0,
    refetchInterval: () => ($strideAddress?.indexOf("stride") === 0 ? 5_000 : false),
    queryFn: async () =>
      await getCosmosChainBalances({
        walletAddress: `${$strideAddress}`,
        url: "https://stride-testnet-api.polkachu.com"
      }),
    select: (data: AwaitedReturnType<typeof getCosmosChainBalances>) =>
      data?.find(balance => balance?.symbol === "ustrd")
  }))
)
</script>

<!-- stride faucet -->
<Card.Root
  class={cn(
    "w-full max-w-lg rounded-lg font-sans border-[#ffffff] bg-cover",
    "bg-[url('/images/backgrounds/stride-background.png')]",
    "bg-[#181825] text-[rgb(60,0,29)] dark:bg-[#2D2D44]/50 dark:text-[rgb(60,0,29)]",
  )}
>
  <Card.Header>
    <Card.Title class="flex justify-between select-none">
      <p class="flex gap-x-3">
        <a target="_blank" rel="noopener noreferrer" href="https://www.stride.zone/">
          <img src="/images/logo/stride/stride-logo.svg" alt="" class="w-18" />
        </a>
        Faucet
      </p>
    </Card.Title>
  </Card.Header>
  <Card.Content>
    {#if $strideFaucetState.kind === "RESULT_OK"}
      <p>
        Tokens sent: <a
          target="_blank"
          rel="noopener noreferrer"
          href={`https://testnet.ping.pub/stride/tx/${$strideFaucetState.message}`}
        >
          <Truncate class="underline" value={$strideFaucetState.message} type="hash" />
        </a>
      </p>
    {:else if $strideFaucetState.kind === "RESULT_ERR"}
      <p class="mb-4">
        {$strideFaucetState.error}
      </p>
      <Button
        class={cn(
          "bg-[rgb(60,0,29)] text-[#ffffff] dark:bg-[rgb(60,0,29)] dark:text-[#ffffff]",
          "disabled:opacity-100 disabled:bg-black/20 rounded-md focus:ring-0 focus-visible:ring-0",
        )}
        on:click={() => strideFaucetState.set({ kind: "IDLE" })}
      >
        Retry
      </Button>
    {:else}
      <form
        action="?"
        method="POST"
        name="faucet-form"
        class="flex flex-col w-full gap-4"
        on:submit|preventDefault|once={requestStrdFromFaucet}
      >
        <div>
          <Label for="address">Address</Label>
          <div class="flex items-start gap-2">
            <div class="w-full">
              <div class="relative w-full mb-2">
                <Input
                  type="text"
                  minlength={44}
                  maxlength={44}
                  readonly={true}
                  required={true}
                  autocorrect="off"
                  id="dydx-address"
                  autocomplete="off"
                  spellcheck="false"
                  autocapitalize="none"
                  value={$strideAddress}
                  data-lpignore={true}
                  data-1p-ignore={true}
                  placeholder="stride14ea6…"
                  name="dydx-wallet-address"
                  class={cn(
                    "bg-[rgb(60,0,29)] text-[#ffffff] dark:bg-[rgb(60,0,29)] dark:text-[#ffffff]",
                    "disabled:opacity-100 disabled:bg-black/20 rounded-md focus:ring-0 focus-visible:ring-0",
                  )}
                  pattern={createCosmosSdkAddressRegex({ prefix: "dydx" }).source}
                />
              </div>
              <div class="flex justify-between px-1">
                <div class="text-xs">
                  <p>
                    {#if $strideAddress?.indexOf("stride") === 0 && $strideBalance.status === "success"}
                      <span>Balance: </span>
                      {$strideBalance?.data?.balance ?? 0}
                      ustrd
                    {:else}
                      Connect cosmos wallet
                    {/if}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="flex flex-row items-center gap-4">
          <Button
            type="submit"
            on:click={event => {
              event.preventDefault()
              requestStrdFromFaucet()
            }}
            disabled={$strideFaucetState.kind !== "IDLE" ||
              isValidBech32Address($strideAddress) === false}
            class={cn(
              "min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50 rounded-md",
              "bg-[#E6007A] text-[#ffffff] dark:bg-[#E6007A] dark:text-[#ffffff]",
            )}
          >
            Submit
            {#if $strideFaucetState.kind !== "IDLE"}
              <span class="ml-2">
                <SpinnerSVG className="w-4 h-4" />
              </span>
            {/if}
          </Button>
          <p class="text-xs">
            STRD faucet is provided by <a
              class="text-[#E6007A]"
              target="_blank"
              rel="noopener noreferrer"
              href="https://www.stride.zone/"
            >
              stride.zone
            </a>
          </p>
        </div>
      </form>
    {/if}
  </Card.Content>
</Card.Root>
