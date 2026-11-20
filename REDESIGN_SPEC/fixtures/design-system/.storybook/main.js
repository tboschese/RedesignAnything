/** Fixture Storybook config — documentary path for DS ingest tests. Not a full Storybook app. */
export default {
  stories: ["../src/**/*.stories.@(js|jsx|ts|tsx)"],
  framework: { name: "@storybook/react-vite", options: {} },
};
