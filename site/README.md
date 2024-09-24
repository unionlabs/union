# Union Site

[union.build](https://union.build) introduces Union and shows our [blog](https://union.build/blog).

## Quickstart

Run the following to start a development server, once it's running edit the files in `site/` and you'll see your changes reflected immediately in the browser.

```sh
nix run .#site-dev-server
```

## Architecture

It's an [Astro] site, hosted by [Vercel] to fetch from our [Contentful] CMS.
Styling is done using [Tailwind]. 3D models are made using [Spline].

[Astro]: https://astro.build
[Tailwind]: https://tailwindcss.com
[Vercel]: https://vercel.com
[Contentful]: https://www.contentful.com
[Spline]: https://spline.design
