<script lang="ts">
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { writable } from "svelte/store"
import { cn } from "$lib/utilities/shadcn.ts"
import { CHAIN_MAP } from "$lib/constants/chains"
import * as Table from "$lib/components/ui/table"
import { createQuery } from "@tanstack/svelte-query"
import { removeArrayDuplicates } from "$lib/utilities"
import { rankItem } from "@tanstack/match-sorter-utils"
import type { Override } from "$lib/utilities/types.ts"
import TimeElapsed from "$lib/components/time-elapsed.svelte"
import { ibcConnectionMapQuery } from "$lib/graphql/documents/ibc-connections"
</script>

<main class={cn('p-4')}></main>

<style lang="postcss"></style>
