<script lang="ts">
  import type { HttpClientError } from "@effect/platform/HttpClientError";
  import type { TimeoutException, UnknownException } from "effect/Cause";
  import type { ParseError } from "effect/ParseResult";
  import type { NoViemChainError } from "$lib/services/evm/clients";
  import type {
    FetchNativeBalanceError,
    ReadContractError,
  } from "$lib/services/evm/balances";
  import type { CreatePublicClientError } from "$lib/services/transfer/errors";
  import type {
    Base64EncodeError,
    QueryBankBalanceError,
  } from "$lib/services/cosmos/balances";
  import type { NoRpcError } from "@unionlabs/sdk/schema";
  import { slide } from "svelte/transition";
  import Button from "$lib/components/ui/Button.svelte";

  interface Props {
    error:
      | UnknownException
      | HttpClientError
      | ParseError
      | TimeoutException
      | NoViemChainError
      | ReadContractError
      | FetchNativeBalanceError
      | CreatePublicClientError
      | QueryBankBalanceError
      | Base64EncodeError
      | NoRpcError;
  }

  let { error }: Props = $props();
  let showDetails = $state(false);

  function getUserFriendlyMessage(error: Props["error"]): string {
    switch (error._tag) {
      case "RequestError":
        return "Unable to connect to the server. Please check your internet connection.";
      case "ResponseError":
        return "The server encountered an error processing your request.";
      case "ParseError":
        return "There was an error processing the data from the server.";
      case "TimeoutException":
        return "The request timed out because it took too long. Please try again.";
      case "UnknownException":
        return "An unexpected error occurred.";
      case "NoViemChain":
        return "Chain configuration not found for the selected network.";
      case "ReadContractError":
        return "Failed to read contract data from the network.";
      case "FetchNativeBalanceError":
        return "Failed to fetch native token balance.";
      case "CreatePublicClientError":
        return "Failed to create network connection.";
      case "QueryBankBalanceError":
        return "Failed to query bank balance from the network.";
      case "Base64EncodeError":
        return "Failed to encode query parameters.";
      case "NoRpcError":
        return `No ${error.type} endpoint available for ${error.chain.display_name}.`;
      default:
        return "Something went wrong. Please try again later.";
    }
  }
</script>

<div class="p-4 rounded bg-red-500 overflow-hidden flex flex-col">
  <div class="flex justify-between gap-2">
    <div>
      <h3 class="text-xl font-bold">Error</h3>
      <p>{getUserFriendlyMessage(error)}</p>
    </div>
    <Button
      variant="secondary"
      class="self-start mt-2"
      onclick={() => (showDetails = !showDetails)}
    >
      {showDetails ? "Hide Details ↑" : "Show Details ↓"}
    </Button>
  </div>

  {#if showDetails}
    <div in:slide out:slide|local={{ delay: 0 }}>
      <section class="mt-4">
        <h3 class="text-lg font-bold">Error Type</h3>
        <pre>{error._tag}</pre>
        <pre class="mt-2 overflow-x-scroll">{error.message}</pre>
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
          <pre class="text-sm">{JSON.stringify(
              error.issue.actual,
              null,
              2,
            )}</pre>
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
          <pre class="text-sm">{JSON.stringify(
              error.chain.rpcs.map((r) => r.type),
              null,
              2,
            )}</pre>
        {/if}
      </section>
    </div>
  {/if}
</div>
