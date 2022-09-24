const path = require('path');

module.exports = {
  baseUrl: '/',
  title: "Something",
  url: "Something.else",
  plugins: [
    [
      '@docusaurus/plugin-content-docs',
      {
        id: 'inx-chronicle-develop',
        path: path.resolve(__dirname, 'docs'),
        routeBasePath: 'chronicle',
        sidebarPath: path.resolve(__dirname, 'sidebars.js'),
        editUrl: 'https://github.com/iotaledger/inx-chronicle/edit/main/documentation',
        remarkPlugins: [require('remark-code-import'), require('remark-import-partial')],
        docItemComponent: "@theme/ApiItem",
        versions: {
          current: {
              label: 'Develop',
              path: 'develop',
              badge: true
          },
        },
      }
    ],
    [
      'docusaurus-plugin-openapi-docs',
      {
        id: "apiDocs",
        docsPluginId: "inx-chronicle-develo",
        config: {
          core: {
            specPath: "https://raw.githubusercontent.com/iotaledger/tips/main/tips/TIP-0025/core-rest-api.yaml",
            outputDir: "docs/reference/apis/core",
            sidebarOptions: {
              groupPathsBy: "tag",
            },
          },
          explorer: {
            specPath: "api/api-explorer.yml",
            outputDir: "docs/reference/apis/explorer",
            sidebarOptions: {
              groupPathsBy: "tag",
            },
          },
          analytics: {
            specPath: "api/api-analytics.yml",
            outputDir: "docs/reference/apis/analytics",
            sidebarOptions: {
              groupPathsBy: "tag",
            },
          },
          indexer: {
            specPath: "https://raw.githubusercontent.com/iotaledger/tips/indexer-api/tips/TIP-0026/indexer-rest-api.yaml",
            outputDir: "docs/reference/apis/indexer",
            sidebarOptions: {
              groupPathsBy: "tag",
            },
          },
        }
      },
    ],
    'docusaurus-theme-openapi-docs',
  ],
  staticDirectories: [path.resolve(__dirname, 'static')],
};
