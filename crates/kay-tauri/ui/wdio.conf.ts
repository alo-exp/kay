import type { Options } from '@wdio/types';

// WebdriverIO v9 Configuration for Kay Tauri Desktop UI smoke tests.
// Uses @crabnebula/tauri-driver for Tauri application automation.

export const config: Options.Testrunner = {
  runner: 'local',
  autoCompile: true,

  specs: ['./e2e/**/*.ts'],

  capabilities: [
    {
      'tauri:options': {
        // The tauri-driver binary is selected automatically based on OS + arch
        // from @crabnebula/tauri-driver-* packages.
        // The app path is resolved relative to this config file.
        application: '../../target/debug/kay-tauri',
      },
    },
  ],

  framework: 'mocha',

  reporters: ['spec'],

  mochaOpts: {
    timeout: 30000,
    ui: 'bdd',
    require: ['source-map-support/register'],
  },

  // Log level for debugging
  logLevels: {
    webdriver: 'info',
    '@wdio/local-runner': 'info',
  },

  // Bail on first failure in CI
  bail: 0,

  // Output directory for test reports
  outputDir: './test-reports',

  // SauceLabs/BrowserStack integration placeholder (for future extension)
  // user, key, and region can be set via env vars for cloud runs.
};