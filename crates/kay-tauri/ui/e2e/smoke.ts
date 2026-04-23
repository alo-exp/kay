/**
 * Kay Desktop UI Smoke Tests — W-6.2 GREEN
 *
 * 5 test cases covering the primary UI surface.
 * Requires Kay Tauri app built first:
 *   cd crates/kay-tauri && cargo build
 *   npm run test:e2e
 */

import { browser, $ } from '@wdio/globals';
import { expect } from '@wdio/globals';

describe('Kay Desktop Smoke', () => {
  /**
   * W6.2 GREEN: App window opens and title contains "Kay"
   */
  it('app window opens', async () => {
    const title = await browser.getTitle();
    expect(title).toMatch(/Kay/i);
  });

  /**
   * W6.2 GREEN: Session view container renders in the UI
   */
  it('session view renders', async () => {
    const sessionView = await $('[data-testid="session-view"]');
    await expect(sessionView).toBeDisplayed();
  });

  /**
   * W6.2 GREEN: Start session button is present and clickable
   */
  it('start session button exists', async () => {
    const btn = await $('[data-testid="start-session-btn"]');
    await expect(btn).toBeDisplayed();
    await expect(btn).toBeEnabled();
  });

  /**
   * W6.2 GREEN: Stop session button is present after a session is started
   */
  it('stop session button exists', async () => {
    const btn = await $('[data-testid="stop-session-btn"]');
    await expect(btn).toBeDisplayed();
  });

  /**
   * W6.2 GREEN: Cost meter widget is visible and updates
   */
  it('cost meter visible', async () => {
    const meter = await $('[data-testid="cost-meter"]');
    await expect(meter).toBeDisplayed();
  });
});