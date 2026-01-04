import { test, expect } from '@playwright/test';

async function waitForWasmHydration(page: import('@playwright/test').Page) {
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);
}

test.describe('Command Palette', () => {
  test('opens when search trigger is clicked', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });

    await page.goto('/');
    await waitForWasmHydration(page);

    if (consoleErrors.length > 0) {
      console.log('Console errors:', consoleErrors);
    }

    const palette = page.locator('.palette');
    await expect(palette).not.toBeVisible();

    const searchTrigger = page.locator('.search-trigger');
    await searchTrigger.click();

    await expect(palette).toBeVisible({ timeout: 5000 });
  });

  test('opens with Ctrl+Shift+K keyboard shortcut', async ({ page }) => {
    await page.goto('/');
    await waitForWasmHydration(page);

    const palette = page.locator('.palette');
    await expect(palette).not.toBeVisible();

    await page.evaluate(() => {
      const event = new KeyboardEvent('keydown', {
        key: 'K',
        code: 'KeyK',
        ctrlKey: true,
        shiftKey: true,
        bubbles: true
      });
      document.dispatchEvent(event);
    });

    await expect(palette).toBeVisible({ timeout: 5000 });
  });
});
