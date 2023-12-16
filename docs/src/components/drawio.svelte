<script lang="ts">
  import { onMount } from 'svelte'
  import diagram from '#/assets/consensus-verification/drawio.svg'

  const htmlElement = document.querySelector('html')
  const currentTheme = htmlElement?.getAttribute('data-theme')
  const selectThemeElement = document.querySelector('select')

  let filter = currentTheme === 'dark' ? 'invert(1)' : 'none'

  function onThemeChange(event: Event) {
    const selectedTheme = event?.target?.value
    filter = selectedTheme === 'dark' ? 'invert(1)' : 'none'
  }

  onMount(() => {
    selectThemeElement?.addEventListener('change', onThemeChange)

    return () => {
      selectThemeElement?.removeEventListener('change', onThemeChange)
    }
  })
</script>

<img
  id="diagram"
  src={diagram.src}
  alt="drawio diagram"
  style:filter
/>
