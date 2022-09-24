/**
 * * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const core_api_sidebar = require('./docs/reference/apis/core/sidebar');
const explorer_api_sidebar = require('./docs/reference/apis/explorer/sidebar');
const analytics_api_sidebar = require('./docs/reference/apis/analytics/sidebar');
const indexer_api_sidebar = require('./docs/reference/apis/indexer/sidebar');

module.exports = {
  docs: [
    {
      type: "doc",
      id: "welcome",
      label: "Welcome",
    },
    {
      type: "category",
      label: "Getting Started",
      collapsed: false,
      items: [
        {
          type: "doc",
          id: "getting_started/docker",
          label: "Docker",
        },
      ],
    },
    {
      type: "category",
      label: "References",
      collapsed: false,
      items: [
        {
          type: "doc",
          id: "reference/authentication",
          label: "Authentication",
        },
        {
          type: "category",
          label: "APIs",
          items: [
            {
              type: "category",
              label: "Core",
              items: core_api_sidebar,
            },
            {
              type: "category",
              label: "Explorer",
              items: explorer_api_sidebar,
            },
            {
              type: "category",
              label: "Analytics",
              items: analytics_api_sidebar,
            },
            {
              type: "category",
              label: "Indexer",
              items: indexer_api_sidebar,
            },
          ],
        },
        {
          type: "doc",
          id: "reference/environment",
          label: "Environment Variables",
        },
      ],
    },
    {
      type: "doc",
      id: "troubleshooting",
      label: "Troubleshooting",
    },
    {
      type: "doc",
      id: "contribute",
      label: "Contribute",
    },
  ],
};
