<script lang="ts">
  import '#/styles/reset.css'
  import '#/styles/tailwind.css'
  import '@fontsource/ibm-plex-mono'
  import '@fontsource/jetbrains-mono'
  import Header from '#/components/Header.svelte'
</script>

<html lang='en'>
  <head>
    <meta charset='UTF8' />
    <meta
      name='viewport'
      content='width=device-width, initial-scale=1.0'
    />
    <title>Union</title>
  </head>
  <body
    class='font-sans flex flex-col items-stretch flex-1 shadow-effect-bg pb-12 bg-black text-white not-content'
  >
    <Header/>

    <slot />

    <footer
      class='flex items-center p-4 gap-4 justify-center text-lg font-normal bottom-0 mx-auto w-full'
    >
      <p>&copy; Union.fi Labs, Inc.</p>
    </footer>
  </body>
</html>


