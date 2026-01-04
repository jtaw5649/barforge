import { test, expect } from '@playwright/test';

test.describe('Stats', () => {
  test('stat icons use inline SVGs for proper color inheritance', async ({ page }) => {
    await page.goto('/');
    const statIcons = page.locator('.stat-icon');
    const count = await statIcons.count();
    expect(count).toBeGreaterThan(0);

    const svgIcons = statIcons.first().locator('svg');
    await expect(svgIcons).toBeVisible();
  });
});
