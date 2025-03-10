<script lang="ts">
import { Option } from "effect"
import { page } from "$app/state"
import { navigation } from "../Sidebar/navigation"
import Button from "../../ui/Button.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { totalErrorCount } from "$lib/stores/app-errors.svelte"

const pageName = $derived(
  Option.fromNullable(
    navigation.find(section => section.items.find(s => s.path === page.url.pathname))
  ).pipe(
    Option.flatMap(s => Option.fromNullable(s.items.find(i => i.path === page.url.pathname))),
    Option.map(s => s.title),
    Option.getOrElse(() => page.url.pathname)
  )
)
</script>

<header class="flex justify-between items-center h-16 px-8 border-b-1 border-zinc-900">
  <h1 class="text-xl font-bold">{pageName}</h1>
  {#if totalErrorCount() > 0}
    <Button variant="danger" onclick={() => uiStore.openErrorsModal()}>
      {totalErrorCount()} Error{totalErrorCount() > 1 ? "s" : ""}
    </Button>
  {/if}
</header>
