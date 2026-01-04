import { test, expect } from '@playwright/test';

test.describe('Footer', () => {
  test('github link uses inline SVG for proper color inheritance', async ({ page }) => {
    await page.goto('/');
    const githubLink = page.locator('.github-link');
    await expect(githubLink).toBeVisible();

    const svgIcon = githubLink.locator('svg.footer-icon');
    await expect(svgIcon).toBeVisible();
  });
});
