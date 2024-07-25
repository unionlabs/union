<script lang="ts">
import { toast } from "svelte-sonner"
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
import { getCosmosChainBalances } from "$lib/queries/balance/cosmos"
import { isValidCosmosAddress } from "$lib/wallet/utilities/validate.ts"
import { cosmosChainAddressTransfers } from "$lib/queries/transfers/cosmos"
import type { AwaitedReturnType, DiscriminatedUnion } from "$lib/utilities/types.ts"
import { convertCosmosAddress, createCosmosSdkAddressRegex } from "$lib/utilities/address.ts"

type DydxFaucetState = DiscriminatedUnion<
  "kind",
  {
    IDLE: {}
    REQUESTING_TOKEN: {}
    SUBMITTING: {}
    RESULT_OK: { message: string }
    RESULT_ERR: { error: string }
  }
>

let dydxAddress = derived(cosmosStore, $cosmosStore =>
  $cosmosStore.address
    ? convertCosmosAddress({
        address: $cosmosStore.address,
        toPrefix: "dydx"
      })
    : ""
)

let dydxFaucetState: Writable<DydxFaucetState> = writable({ kind: "IDLE" })

const requestDydxFromFaucet = async () => {
  if ($dydxFaucetState.kind === "IDLE" || $dydxFaucetState.kind === "REQUESTING_TOKEN") {
    dydxFaucetState.set({ kind: "SUBMITTING" })
  }

  if ($dydxFaucetState.kind === "SUBMITTING") {
    try {
      const response = await fetch("https://faucet.v4testnet.dydx.exchange/faucet/native-token", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ address: $dydxAddress })
      })
      if (!response.ok) {
        const error = (await response.json()) as { error: string }
        dydxFaucetState.set({
          kind: "RESULT_ERR",
          error: error?.error?.includes("many")
            ? "Rate limit exceeded"
            : error?.error ?? "Unknown error"
        })
        return
      }

      dydxFaucetState.set({
        kind: "RESULT_OK",
        message: "success"
      })
      return true
    } catch (error) {
      dydxFaucetState.set({
        kind: "RESULT_ERR",
        error: `Faucet error: ${error}`
      })
      return
    }
  }
}

let dydxBalance = createQuery(
  derived(dydxAddress, $dydxAddress => ({
    queryKey: ["dydx-balance", $dydxAddress],
    enabled: $dydxAddress?.indexOf("dydx") === 0,
    refetchInterval: () => ($dydxAddress?.indexOf("dydx") === 0 ? 5_000 : false),
    queryFn: async () =>
      await getCosmosChainBalances({
        walletAddress: `${$dydxAddress}`,
        url: "https://dydx-testnet-api.polkachu.com"
      }),
    select: (data: AwaitedReturnType<typeof getCosmosChainBalances>) =>
      data?.find(balance => balance?.symbol === "adv4tnt")
  }))
)

let dydxAddressTransfers = createQuery(
  derived(dydxAddress, $dydxAddress => ({
    queryKey: ["dydx-address-transfers", $dydxAddress],
    enabled: $dydxAddress?.indexOf("dydx") === 0,
    refetchInterval: () => ($dydxAddress?.indexOf("dydx") === 0 ? 5_000 : false),
    queryFn: async () =>
      cosmosChainAddressTransfers({
        address: $dydxAddress,
        include: ["recipient"],
        url: "https://dydx-testnet-api.polkachu.com"
      }),
    select: (data: AwaitedReturnType<typeof cosmosChainAddressTransfers>) =>
      data?.tx_responses
        .filter(
          txResponse =>
            txResponse.code === 0 &&
            txResponse.tx.body.messages.some(
              // faucet address
              message =>
                message.from_address === "dydx1g2ygh8ufgwwpg5clp2qh3tmcmlewuyt2z6px8k" &&
                message.amount.at(0)?.denom === "adv4tnt"
            )
        )
        .at(-1)
  }))
)

$: console.info($dydxAddressTransfers?.data)
</script>

<!-- dydx faucet -->
<Card.Root
  class="w-full max-w-lg bg-[#181825] text-[#FFFFFF] rounded-lg font-sans bg-[url('https://dydx.exchange/dots.svg')]"
>
  <Card.Header>
    <Card.Title class="flex justify-between select-none">
      <p class="flex gap-x-3">
        <img src="https://dydx.exchange/logo.svg" alt="" class="w-14" />
        Faucet
      </p>
      <img alt="" src="https://dydx.exchange/icon.svg" class="w-6" />
    </Card.Title>
  </Card.Header>
  <Card.Content>
    {#if $dydxFaucetState.kind === "RESULT_OK"}
      <p>
        Tokens sent: <a
          href={`https://www.mintscan.io/dydx-testnet/tx/${$dydxFaucetState.message}`}
        >
          <Truncate
            class="underline"
            value={$dydxFaucetState.message}
            type="hash"
          />
        </a>
      </p>
    {:else if $dydxFaucetState.kind === "RESULT_ERR"}
      <p class="mb-4">
        {$dydxFaucetState.error}
      </p>
      <Button on:click={() => dydxFaucetState.set({ kind: "IDLE" })}>
        Retry
      </Button>
    {:else}
      <form
        action="?"
        method="POST"
        name="faucet-form"
        class="flex flex-col w-full gap-4"
        on:submit|preventDefault|once={requestDydxFromFaucet}
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
                  value={$dydxAddress}
                  data-lpignore={true}
                  data-1p-ignore={true}
                  placeholder="dydx14ea6…"
                  name="dydx-wallet-address"
                  class="disabled:opacity-100 disabled:bg-black/20 bg-[#2D2D44] text-[#ffffff] rounded-md"
                  pattern={createCosmosSdkAddressRegex({ prefix: "dydx" })
                    .source}
                />
              </div>
              <div class="flex justify-between px-1 text-white">
                <div class="text-xs">
                  <p>
                    {#if $dydxAddress?.indexOf("dydx") === 0 && $dydxBalance.status === "success"}
                      <span>Balance: </span>
                      {$dydxBalance?.data?.balance ?? 0}
                      adv4tnt
                    {:else}
                      Connect cosmos wallet
                    {/if}
                  </p>
                </div>
                <!-- {#if dydxAddress !== convertCosmosAddress( { address: `${$cosmosStore.address}`, toPrefix: "dydx" } )}
                    <button
                      type="button"
                      on:click={resetDydxInput}
                      class="text-xs text-muted-foreground hover:text-primary transition"
                    >
                      Reset
                    </button>
                  {/if} -->
              </div>
            </div>
          </div>
        </div>
        <div class="flex flex-row items-center gap-4">
          <Button
            type="submit"
            on:click={async (event) => {
              event.preventDefault()
              const success = await requestDydxFromFaucet()
              if (!success) {
                toast.loading("Faucet request submitted", {
                  class: "text-md",
                  dismissable: true,
                  duration: Number.POSITIVE_INFINITY,
                  actionButtonStyle: "font-mono text-xl",
                  description: "Waiting for transaction receipt",
                  action: {
                    label: "explorer  🔗",
                    onClick: async (event) =>
                      window.open(
                        `https://mintscan.io/dydx-testnet/address/${$dydxAddress}`,
                        "_blank",
                        "noopener"
                      )
                  }
                })
              }
            }}
            disabled={$dydxFaucetState.kind !== "IDLE" ||
              isValidCosmosAddress($dydxAddress, ["dydx"]) === false}
            class={cn(
              "min-w-[110px] disabled:cursor-not-allowed disabled:opacity-50 bg-[#6866FF] rounded-md"
            )}
          >
            Submit
            {#if $dydxFaucetState.kind !== "IDLE"}
              <span class="ml-2">
                <SpinnerSVG className="w-4 h-4" />
              </span>
            {/if}
          </Button>
          <p class="text-xs">
            dYdX faucet is provided by <a
              class="text-[#9e9dff]"
              target="_blank"
              rel="noopener noreferrer"
              href="https://dydx.exchange/"
            >
              dydx.exchange
            </a>
          </p>
        </div>
      </form>
    {/if}
  </Card.Content>
</Card.Root>

<style lang="postcss"></style>
