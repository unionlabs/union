<script lang="ts">
import { onDestroy } from "svelte"
import { supabase } from "$lib/supabase/client"

let live = $state<Array<any>>([])

const handleInserts = (payload: { new: any }) => {
  console.log(payload)
  live.push(payload)
}

const subscription = supabase
  .channel("table_db_changes")
  .on(
    "postgres_changes",
    {
      event: "INSERT",
      schema: "public",
      table: "log"
    },
    handleInserts
  )
  .subscribe()

onDestroy(() => {
  subscription.unsubscribe()
})
</script>

<div class="flex-col">
  {#each live as item (item.created_at)}
    <div class="text-white">{JSON.stringify(item)}</div>
  {/each}
</div>