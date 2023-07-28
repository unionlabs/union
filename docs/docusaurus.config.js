// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require("prism-react-renderer/themes/github");
const darkCodeTheme = require("prism-react-renderer/themes/dracula");

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: "Union Finance",
  tagline: "Connecting blockchains trustlessly",

  markdown: {
    mermaid: true,
  },
  themes: ["@docusaurus/theme-mermaid"],
  // Set the production url of your site here
  url: "https://docs.union.build",

  baseUrl: "/",

  organizationName: "unionlabs",
  projectName: "union",

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          routeBasePath: "/",
          sidebarPath: require.resolve("./sidebars.js"),
          editUrl: "https://github.com/unionlabs/union/edit/main/docs",
        },
        blog: false,
        theme: {
          customCss: require.resolve("./src/css/custom.css"),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      colorMode: {
        defaultMode: "dark",
        respectPrefersColorScheme: true,
      },
      // Replace with your project's social card
      // image: 'img/docusaurus-social-card.jpg',
      navbar: {
        title: "Union",
        logo: {
          alt: "Logo",
          src: "img/union-logo.svg",
        },
        items: [],
      },
      footer: {
        style: "dark",
        links: [
          // {
          //   title: "Community",
          //   items: [
          //     {
          //       label: "Twitter",
          //       href: "https://twitter.com/unionfin",
          //     },
          //   ],
          // },
          {
            items: [
              {
                label: "Website",
                href: "https://union.build",
              },
              {
                label: "GitHub",
                href: "https://github.com/unionlabs",
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Union.fi Labs, Inc.`,
      },
      prism: {
        theme: lightCodeTheme,
        darkTheme: darkCodeTheme,
        additionalLanguages: ["nix"],
      },
    }),
};

module.exports = config;
