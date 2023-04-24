// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  tutorialSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Validators & Nodes',
      items: [
        'nodes-validators/setting-up-node'],
    },
  ],
};

module.exports = sidebars;
