/**
 * Kay Desktop UI Smoke Tests
 *
 * RED stub phase: 5 it.skip() test cases covering the primary UI surface.
 * These will be implemented in GREEN phase once the Tauri UI components
 * are wired up with data-testid attributes.
 *
 * NOTE: These tests require the Kay Tauri app to be built first:
 *   cd crates/kay-tauri && cargo build
 *   npm run test:e2e
 */

import { browser, $ } from '@wdio/globals';
import { expect } from '@wdio/globals';

describe('Kay Desktop Smoke', () => {
  /**
   * W6.1 RED: App window opens and title contains "Kay"
   */
  it.skip('app window opens', async () => {
    const title = await browser.getTitle();
    expect(title).toMatch(/Kay/i);
  });

  /**
   * W6.1 RED: Session view container renders in the UI
   */
  it.skip('session view renders', async () => {
    const sessionView = await $('[data-testid="session-view"]');
    await expect(sessionView).toBeDisplayed();
  });

  /**
   * W6.1 RED: Start session button is present and clickable
   */
  it.skip('start session button exists', async () => {
    const btn = await $('[data-testid="start-session-btn"]');
    await expect(btn).toBeDisplayed();
    await expect(btn).toBeEnabled();
  });

  /**
   * W6.1 RED: Stop session button is present after a session is started
   */
  it.skip('stop session button exists', async () => {
    const btn = await $('[data-testid="stop-session-btn"]');
    await expect(btn).toBeDisplayed();
  });

  /**
   * W6.1 RED: Cost meter widget is visible and updates
   */
  it.skip('cost meter visible', async () => {
    const meter = await $('[data-testid="cost-meter"]');
    await expect(meter).toBeDisplayed();
  });
});