<script lang="ts">
  /**
   * Local filepath of the file to download.
   * If there's a static file you should store it in `/public/<filename>`,
   * and then pass the filepath as `/<filename>`.
   */
  export let filepath: string
  export let filename: string | undefined

  filename = filename || filepath.split('/').pop()
</script>

<button class="rounded-sm">
  <a
    href={filepath}
    download={filename}
  >
    {filename}
  </a>
</button>
