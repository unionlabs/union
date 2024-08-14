# Union Site

[union.build](https://union.build) introduces Union, hosts our [docs](https://union.build/docs), and shows our [blog](https://union.build/blog). 


## Quickstart

Run the following to start a development server, once it's running edit the files in `site/` and you'll see your changes reflected immediately in the browser.

```
nix run .#site-dev-server
```

## Architecture

It's an [Astro] site, hosted by [Netlify] to fetch from our [Contentful] CMS. The docs are built using [Starlight]. Styling is done using [Tailwind]. 3D models are made using [Spline].

[Astro]: https://astro.build
[Tailwind]: https://tailwindcss.com
[Starlight]: https://starlight.astro.build
[Netlify]: https://www.netlify.com
[Contentful]: https://www.contentful.com
[Spline]: https://spline.design
