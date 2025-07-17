<script lang="ts">
import { navigation } from "./config"
import NavSocialLink from "./NavSocialLink.svelte"

let { expanded = true }: { expanded?: boolean } = $props()

const moreUnionSection = navigation.find(section => section.title === "More Union")
</script>

<footer class="px-2 pb-3">
  <div class="flex flex-col items-center transition-all duration-300">
    <div
      class="flex justify-center gap-2 transition-all duration-300"
      class:opacity-100={expanded}
      class:opacity-0={!expanded}
      class:h-8={expanded}
      class:h-0={!expanded}
      class:overflow-hidden={!expanded}
    >
      {#if moreUnionSection}
        {#each moreUnionSection.items as { title, icon, path }}
          <NavSocialLink
            name={title}
            href={path}
          >
            <svelte:component
              this={icon}
              class="w-4 h-4"
            />
          </NavSocialLink>
        {/each}
      {/if}
    </div>
  </div>
</footer>
