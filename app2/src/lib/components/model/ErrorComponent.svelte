<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { FetchAptosTokenBalanceError } from "$lib/services/aptos/balances"
import type { QueryBankBalanceError } from "$lib/services/cosmos/balances"
import type { FetchNativeBalanceError, ReadContractError } from "$lib/services/evm/balances"
import type { NoViemChainError } from "$lib/services/evm/clients"
import type {
  CosmosWalletNotConnectedError,
  CosmosWalletNotOnWindowError,
  CosmWasmError,
  GasPriceError,
  GetChainInfoError,
  NoCosmosChainInfoError,
  OfflineSignerError,
} from "$lib/services/transfer-ucs03-cosmos"
import type { WaitForTransactionReceiptError } from "$lib/services/transfer-ucs03-evm"
import type {
  AmountError,
  ConnectorClientError,
  CreatePublicClientError,
  CreateWalletClientError,
  SwitchChainError,
} from "$lib/services/transfer/errors"
import type { Base64EncodeError } from "$lib/utils/base64"
import type { FromHexError } from "$lib/utils/hex"
import type { HttpClientError } from "@effect/platform/HttpClientError"
import type { ExecuteContractError } from "@unionlabs/sdk/cosmos"
import {
  CreateViemPublicClientError,
  CreateViemWalletClientError,
  WriteContractError,
} from "@unionlabs/sdk/evm"
import type {
  CosmosAddressEncodeError,
  NoRpcError,
  NotACosmosChainError,
} from "@unionlabs/sdk/schema"
import { CryptoError, extractErrorDetails } from "@unionlabs/sdk/utils"
import { Match, pipe } from "effect"
import type { NoSuchElementException, TimeoutException, UnknownException } from "effect/Cause"
import type { ParseError } from "effect/ParseResult"
import { slide } from "svelte/transition"
import BaselineCloseIcon from "../icons/BaselineCloseIcon.svelte"
import SharpDownloadIcon from "../icons/SharpDownloadIcon.svelte"
import SharpErrorOutlineIcon from "../icons/SharpErrorOutlineIcon.svelte"
import SharpOpenInBrowserIcon from "../icons/SharpOpenInBrowserIcon.svelte"
import Modal from "../ui/Modal.svelte"
import Tooltip from "../ui/Tooltip.svelte"

interface Props {
  error:
    | AmountError
    | Base64EncodeError
    | ConnectorClientError
    | CosmWasmError
    | CosmosAddressEncodeError
    | CosmosWalletNotConnectedError
    | CosmosWalletNotOnWindowError
    | CreatePublicClientError
    | CreateViemPublicClientError
    | CreateViemWalletClientError
    | CreateWalletClientError
    | CryptoError
    | ExecuteContractError
    | FromHexError
    | FetchNativeBalanceError
    | GasPriceError
    | GetChainInfoError
    | HttpClientError
    | NoCosmosChainInfoError
    | NoRpcError
    | NoSuchElementException
    | NoViemChainError
    | NotACosmosChainError
    | OfflineSignerError
    | ParseError
    | QueryBankBalanceError
    | ReadContractError
    | SwitchChainError
    | TimeoutException
    | FetchAptosTokenBalanceError
    | UnknownException
    | WaitForTransactionReceiptError
    | WriteContractError
  onOpen?: () => void | undefined
  onClose?: (() => void) | undefined
}

let { error, onClose, onOpen }: Props = $props()
let showDetails = $state(false)
let visible = $state(true)

const getUserFriendlyMessage = pipe(
  Match.type<Props["error"]>(),
  Match.tags({
    AmountError: () => "Amount invalid.",
    Base64EncodeError: () => "Failed to encode query parameters.",
    ConnectorClientError: (x) => `A connector client error occurred: ${x.message}`,
    CosmWasmError: (x) => `CosmWasm failure: ${x.message}`,
    CosmosAddressEncodeError: (x) => `Failed to encode the Cosmos address ${x.address}.`,
    CosmosSwitchChainError: (x) =>
      `Failed to switch to chain ${x.chainInfo?.chainName}. Please switch manually within wallet.`,
    CosmosWalletNotConnectedError: () =>
      `Cosmos wallet not connected. Please check wallet connection.`,
    CosmosWalletNotOnWindowError: (x) => `${x.kind} not found on window. Please check wallet.`,
    CreatePublicClientError: () => "Failed to create network connection.",
    CreateViemPublicClientError: (x) => `Could not create the EVM public client: ${x.message}`,
    CreateViemWalletClientError: (x) => `Could not create the EVM wallet client: ${x.message}.`,
    CreateWalletClientError: (x) => `Could not create the wallet client: ${x.message}.`,
    CryptoError: () => `Browser does not support cryptography functions.`,
    EvmSwitchChainError: () => `Failed to switch chain. Please switch manually within wallet.`,
    ExecuteContractError: (x) => `Failed to execute contract: ${(x.cause as Error).message}`, // XXX: improve error type
    FetchNativeBalanceError: () => "Failed to fetch native token balance.",
    GasPriceError: () => `Incorrect gas price configuration.`,
    FromHexError: () => `Failed to decode hex.`,
    GetChainInfoError: (x) => `No info for EVM chain ${x.chainId}.`, // TODO: rename to EVM
    NoCosmosChainInfoError: (x) => `No info for Cosmos chain ${x.chain.display_name}.`,
    FetchAptosTokenBalanceError: () => `Failed to fetch aptos token balance.`,
    NoRpcError: (error) => `No ${error.type} endpoint available for ${error.chain.display_name}.`,
    NoSuchElementException: () => "An unexpected error occurred.", // TODO: remove me for more explicit errors
    NoViemChain: () => "Chain configuration not found for the selected network.",
    NotACosmosChainError: () => "The selected chain is not a Cosmos chain.",
    OfflineSignerError: (x) => `Wallet failed to provide offline signer for ${x.chain_id}.`,
    ParseError: () => "There was an error processing the data from the server.",
    QueryBankBalanceError: () => "Failed to query bank balance from the network.",
    ReadContractError: () => "Failed to read contract data from the network.",
    RequestError: () => "Unable to connect to the server. Please check your internet connection.",
    ResponseError: () => "The server encountered an error processing your request.",
    TimeoutException: () => "The request timed out because it took too long. Please try again.",
    UnknownException: () => "An unexpected error occurred.",
    WaitForTransactionReceiptError: (x) =>
      `Waiting for the transaction receipt failed: ${x.message}`,
    WriteContractError: (e) =>
      `Failed to write to the contract: ${(e.cause.cause as any).shortMessage}`, // TODO: improve error type
  }),
  Match.orElse((x) => `Unexpected error: ${x?.["_tag"]}`),
)

const _writeToClipboard = () => {
  navigator.clipboard.writeText(JSON.stringify(extractErrorDetails(error), null, 2))
}

const exportData = () => {
  const datetime = new Date().toISOString().replace(/-|:|\.\d+/g, "")
  const data = JSON.stringify(extractErrorDetails(error), null, 2)
  const blob = new Blob([data], { type: "application/json" })
  const url = window.URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = url
  anchor.download = `union-log-${datetime}.json`
  anchor.click()
  window.URL.revokeObjectURL(anchor.href)
}

const onShowDetails = () => {
  if (onOpen) {
    onOpen()
  } else {
    showDetails = !showDetails
  }
}
</script>

{#if visible}
  <div class="p-4 rounded bg-zinc-925 border-2 border-red-500 overflow-hidden flex flex-col">
    {#if onClose}
      <div class="flex flex-row mb-2">
        <SharpErrorOutlineIcon class="text-red-500 size-4 min-w-4" />
        <div class="grow"></div>
        <Button
          class="self-end p-0 h-4"
          variant="outline"
          onclick={onClose}
        >
          <BaselineCloseIcon
            height="1rem"
            width="1rem"
          />
        </Button>
      </div>
    {/if}
    <div class="flex justify-between items-center gap-2">
      {#if !onClose}
        <SharpErrorOutlineIcon class="text-red-500 size-4 min-w-4" />
      {/if}
      <p>{getUserFriendlyMessage(error)}</p>
      <div class="grow"></div>
      <Tooltip delay={"quick"}>
        {#snippet trigger()}
          <Button
            variant="secondary"
            onclick={onShowDetails}
          >
            <SharpOpenInBrowserIcon class="size-4" />
          </Button>
        {/snippet}

        {#snippet content()}
          Open Details
        {/snippet}
      </Tooltip>
      <!-- <Tooltip delay={"quick"}>
      {#snippet trigger()}
        <Button variant="secondary" onclick={writeToClipboard}>
          <SharpContentCopyIcon class="size-4" />
        </Button>
      {/snippet}
      {#snippet content()}
        Copy to Clipboard
      {/snippet}
    </Tooltip> -->
      {#if !onOpen}
        <Tooltip delay={"quick"}>
          {#snippet trigger()}
            <Button
              variant="primary"
              onclick={exportData}
            >
              <SharpDownloadIcon class="size-4" />
            </Button>
          {/snippet}
          {#snippet content()}
            Download Log
          {/snippet}
        </Tooltip>
      {/if}
    </div>

    <Modal
      isOpen={showDetails}
      onClose={() => (showDetails = !showDetails)}
      class="w-full max-w-4xl"
    >
      <div
        class="overflow-auto mt-6"
        in:slide
        out:slide|local={{ delay: 0 }}
      >
        <section class="mt-4">
          <h3 class="text-lg font-bold">Error Type</h3>
          <pre>{error._tag}</pre>
          <pre class="mt-2">{error.message}</pre>
        </section>

        {#if error.cause}
          <section class="mt-4">
            <h3 class="text-lg font-bold">Cause</h3>
            <pre>{error.cause}</pre>
          </section>
        {/if}

        {#if error.stack}
          <section class="mt-4">
            <h3 class="text-lg font-bold">Stack</h3>
            <pre class="text-sm">{error.stack}</pre>
          </section>
        {/if}

        <section class="mt-4">
          <h3 class="text-lg font-bold">Additional Details</h3>
          {#if error._tag === "RequestError"}
            <p>{error.description}</p>
            <p>Method and URL: {error.methodAndUrl}</p>
          {:else if error._tag === "ResponseError"}
            <p>{error.description}</p>
            <p>Method and URL: {error.methodAndUrl}</p>
          {:else if error._tag === "ParseError"}
            <p>Actual data that was parsed:</p>
            <pre
              class="text-sm"
            >
{JSON.stringify(
                error.issue.actual,
                null,
                2,
              )}</pre
            >
          {:else if error._tag === "UnknownException"}
            <p>This is an unknown exception. Full details here:</p>
            <pre class="text-sm">{JSON.stringify(error, null, 2)}</pre>
          {:else if error._tag === "NoViemChain"}
            <p>Chain ID: {error.chain.chain_id}</p>
            <p>Universal Chain ID: {error.chain.universal_chain_id}</p>
          {:else if error._tag === "ReadContractError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "FetchNativeBalanceError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "CreatePublicClientError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "QueryBankBalanceError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "Base64EncodeError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "NoRpcError"}
            <p>Chain: {error.chain.display_name}</p>
            <p>RPC Type: {error.type}</p>
            <p>Available RPC types:</p>
            <pre
              class="text-sm"
            >
{JSON.stringify(
                error.chain.rpcs.map((r) => r.type),
                null,
                2,
              )}</pre
            >
          {/if}
        </section>
      </div>
    </Modal>
  </div>
{/if}
