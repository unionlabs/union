<script lang="ts">
import { Debounced } from "runed"
import { dedent } from "ts-dedent"
import { cn } from "#/lib/shadcn.ts"
import curlSvg from "#/assets/icons/curl.svg?raw"
import JsonIcon from "icons:svelte/mdi/code-json"
import { highlightCode } from "#/lib/highlight-code.ts"
import * as Table from "#/components/svelte/ui/table/index.ts"
import * as Pagination from "#/components/svelte/ui/pagination/index.ts"
import { graphqlQueryToCurl, splitArray, stringIsJSON } from "#/lib/utilities.ts"
import GraphqlPlaygroundLink from "#/components/svelte/graphql-playground-link.svelte"
import deployments from "~root/deployments/deployments.json" with { type: "json" }
</script>

{#each deployments as network}
  <h3>
    {network.chain_id} Deployments
  </h3>
  <Table.Root class="w-full border border-neutral-500 rounded-sm">
    <Table.Row class="w-full">
      <Table.Cell>
        Category
      </Table.Cell>
      <Table.Cell>
        Name
      </Table.Cell>
      <Table.Cell>
        Address
      </Table.Cell>
    </Table.Row>
    <Table.Row class="w-full">
      <Table.Cell class="w-full">
        <b>Core</b>
      </Table.Cell>
      <Table.Cell>
        ibc-union-core
      </Table.Cell>
      <Table.Cell>
        {network.deployments.core.address}
      </Table.Cell>
    </Table.Row>
    {#each Object.entries(network.deployments.lightclient) as [name, data], index}
      <Table.Row>
        <Table.Cell class="w-full">
          {#if index === 0}
            <b>Light-Client</b>
          {/if}
        </Table.Cell>
        <Table.Cell>
          {name}
        </Table.Cell>
        <Table.Cell>
          {data.address}
        </Table.Cell>
      </Table.Row>
    {/each}
    {#if Object.keys(network.deployments.app).length !== 0 }
      {#each Object.entries(network.deployments.app) as [name, data], index}
        <Table.Row>
          <Table.Cell class="w-full">
            {#if index === 0}
              <b>App</b>
            {/if}
          </Table.Cell>
          <Table.Cell>
            {name}
          </Table.Cell>
          <Table.Cell>
            {data.address}
          </Table.Cell>
        </Table.Row>
      {/each}
    {/if}
  </Table.Root>
{/each}
