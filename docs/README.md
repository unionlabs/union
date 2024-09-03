# Union Docs

[![Built with Starlight](https://astro.badg.es/v2/built-with-starlight/tiny.svg)](https://starlight.astro.build)

[docs.union.build](https://docs.union.build) hosts our [docs](https://docs.union.build).

## Quickstart

Run the following to start a development server, once it's running edit the files in `site/` and you'll see your changes reflected immediately in the browser.

```sh
nix run .#docs-dev-server
```

## Architecture

It's an [Astro] site, hosted on [Cloudflare Pages]. The docs are built using [Starlight]. Styling is done using [Tailwind].

[Astro]: https://astro.build
[Tailwind]: https://tailwindcss.com
[Starlight]: https://starlight.astro.build
[Cloudflare Pages]: https://pages.cloudflare.com
