<script lang="ts">
import type { CreateQueryResult } from "@tanstack/svelte-query"
import LoadingLogo from "./loading-logo.svelte"
import ExplorerPagination from "./explorer-pagination.svelte"
import type { Readable } from "svelte/store"
import type { ColumnDef } from "@tanstack/svelte-table"
import Table from "../../routes/explorer/(components)/table.svelte"

type DataRow = $$Generic

interface Props {
  queryResult: CreateQueryResult
  columns: Array<ColumnDef<DataRow & { timestamp: string }>>
  dataStore: Readable<Array<DataRow & { timestamp: string }>>
}

let { queryResult, columns, dataStore }: Props = $props()
</script>


{#if $queryResult.data}
  <Table {dataStore} {columns} />
  <!-- <ExplorerPagination explorerItems={dataStore} /> -->
{:else if $queryResult.status  === "pending"}
  <LoadingLogo class="size-16" />
{/if}
