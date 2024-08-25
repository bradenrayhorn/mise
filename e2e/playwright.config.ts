import { defineConfig, devices } from '@playwright/test';

/**
 * See https://playwright.dev/docs/test-configuration.
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'html' : 'list',
  use: {
    baseURL: 'http://127.0.0.1:3000',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },

    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 12'] },
    },
  ],

  /* Run your local dev server before starting the tests */
  webServer: [
    {
      command: 'fake-oidc',
      url: 'http://127.0.0.1:7835/health',
      reuseExistingServer: !process.env.CI,
    },
    {
      command:
        '(cd ../ui && npm run build) && (cd ../server && MISE_CONFIG=../e2e/mise.toml cargo run --release)',
      url: 'http://127.0.0.1:3000/health-check',
      reuseExistingServer: !process.env.CI,
      stdout: 'pipe',
      timeout: 180000,
    },
  ],
});
