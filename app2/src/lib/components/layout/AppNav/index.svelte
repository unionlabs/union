<script lang="ts">
  import NavFooter from './NavFooter.svelte'
  import NavLogo from './NavLogo.svelte'
  import NavMenu from './NavMenu.svelte'
  import ProfileCard from './ProfileCard.svelte'

  let expanded = $state(false)
  let timeout: NodeJS.Timeout

  function enter() {
    timeout = setTimeout(() => {
      expanded = true
    }, 200)
  }

  function leave() {
    clearTimeout(timeout)
    expanded = false
  }
</script>

<nav
  onmouseenter={enter}
  onmouseleave={leave}
  class="fixed z-100 hidden h-full flex-col overflow-hidden border-r border-zinc-800 bg-zinc-950 p-2 pr-1.5 text-zinc-400 transition-[width] duration-300 ease-in-out md:flex {expanded ? 'w-64' : 'w-16'}"
>
  <div class="flex h-full flex-col text-sm font-medium">
    <!-- Logo Section -->
    <div class="flex-shrink-0">
      <NavLogo {expanded} />
    </div>
    
    <div class="pt-4 pb-2">
      <ProfileCard {expanded} />
    </div>
    <!-- Main Content Area -->
    <div class="flex flex-1 flex-col justify-between">
      <NavMenu {expanded} />
      <NavFooter {expanded} />
    </div>
  </div>
</nav>
