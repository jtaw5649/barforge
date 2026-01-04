import { test, expect } from '@playwright/test';

test.describe('Search Shortcut', () => {
  test('search trigger displays kbd elements for keyboard shortcut', async ({ page }) => {
    await page.goto('/');
    const searchShortcut = page.locator('.search-shortcut');
    await expect(searchShortcut).toBeVisible();

    const kbdElements = searchShortcut.locator('kbd');
    const count = await kbdElements.count();
    expect(count).toBe(3);
  });
});
