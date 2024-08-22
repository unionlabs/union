<script lang="ts">
import type { CreateQueryResult } from "@tanstack/svelte-query"
import LoadingLogo from "./loading-logo.svelte"
import ExplorerPagination from "./explorer-pagination.svelte"
import type { Readable } from "svelte/store"
import type { ColumnDef } from "@tanstack/svelte-table"
import Table from "../../routes/explorer/(components)/table.svelte"

type DataRow = $$Generic

export let queryResult: CreateQueryResult
export let columns: Array<ColumnDef<DataRow & { timestamp: string }>>
export let dataStore: Readable<Array<DataRow & { timestamp: string }>>
</script>


{#if $queryResult.data}
  <Table {dataStore} {columns} />
  <ExplorerPagination explorerItems={dataStore} />
{:else if $queryResult.status  === "pending"}
  <LoadingLogo class="size-16" />
{/if}
