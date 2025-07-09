<script lang="ts">
  import { navigation } from './config'
  import { uiStore } from '$lib/stores/ui.svelte'
  import NavLinkActive from './NavLinkActive.svelte'
  import ExpandableSection from './ExpandableSection.svelte'

  let { expanded = false }: { expanded?: boolean } = $props()
  
  // Track which main items have their subroutes expanded
  let expandedItems = $state<Set<string>>(new Set())
  
  // Close all expanded items when sidebar collapses
  $effect(() => {
    if (!expanded) {
      expandedItems = new Set()
    }
  })
  
  function toggleSubroutes(itemPath: string) {
    if (expandedItems.has(itemPath)) {
      expandedItems.delete(itemPath)
    } else {
      expandedItems.add(itemPath)
    }
    expandedItems = new Set(expandedItems) // Trigger reactivity
  }
</script>

<div class="flex min-h-[60%] flex-col space-y-1 overflow-x-hidden pb-4">
  {#each navigation as section}
    {#if section.title !== "Developer" || uiStore.showDeveloperPages}
      {#if section.title !== "More Union"}
        {#each section.items as item}
          {#if item.subroutes && item.subroutes.length > 0}
            <!-- Expandable section -->
            <ExpandableSection
              {item}
              {expanded}
              isExpanded={expandedItems.has(item.path)}
              onToggle={() => toggleSubroutes(item.path)}
            />
          {:else}
            <!-- Direct navigation link -->
            <NavLinkActive page={{
              name: item.title,
              icon: item.icon,
              href: item.path,
              aliases: [item.path],
              external: item.external ?? false,
              new: false
            }}>
              {#snippet children(size)}
                <svelte:component this={item.icon} class={size} />
              {/snippet}
            </NavLinkActive>
          {/if}
        {/each}
      {/if}
    {/if}
  {/each}
</div>
