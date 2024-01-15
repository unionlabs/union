<script lang="ts">
  import '@fontsource/ibm-plex-mono'
  import '@fontsource/jetbrains-mono'
  import '#/styles/index.css'
  import '#/styles/tailwind.css'
  import Header from '#/components/header.svelte'
  import XLogo from '#/assets/x.svg'
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
    class='font-sans flex flex-col min-h-screen grid-effect-bg flex-1 shadow-effect-bg pb-12'
  >
    <Header/>
    <slot />
    <footer
      class='flex items-center p-4 gap-4 justify-center text-lg font-extrabold text-gray-200 bottom-0 mx-auto w-full'
    >
      <p>&copy; Union.fi Labs, Inc.</p>
    </footer>
  </body>
</html>

<style>
  *::-webkit-scrollbar {
    height: 0.3rem;
    width: 0rem;
  }

  :global(body) {
    background-color: theme(colors.black);
    color: #ffffff;
  }

  :global(a),
  :global(a:visited) {
    color: white;
    text-decoration: none;
  }

  .grid-effect-bg {
    --grid-size: 4rem;
    @media (min-width: 1024px) {
      --grid-size: 6rem;
    }
    --grid-line-color: rgba(20, 20, 20, 1);

    background: black;
    background-image: radial-gradient(rgba(255, 135, 195, 0.02), rgba(0, 0, 0, 1)),
      linear-gradient(var(--grid-line-color) 0.1em, transparent 0.1em),
      linear-gradient(90deg, var(--grid-line-color) 0.1em, transparent 0.1em);
    background-position: center center;
    background-size:
      cover,
      var(--grid-size) var(--grid-size),
      var(--grid-size) var(--grid-size);
  }
</style>
