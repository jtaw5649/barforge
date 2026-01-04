import { test, expect } from '@playwright/test';

test.use({ viewport: { width: 1920, height: 1080 } });

test('modules search list view persists when navigating from modules tab', async ({ page }) => {
  await page.goto('/modules');
  await page.waitForLoadState('networkidle');

  const tablist = page.getByRole('tablist', { name: 'Browse navigation' });
  const searchTab = tablist.getByRole('tab', { name: 'Search' });
  await searchTab.click();
  await expect(page).toHaveURL(/\/modules\/search/);
  await page.waitForLoadState('networkidle');

  const results = page.locator('.results');
  await expect(results).toBeVisible();

  const listButton = page.getByRole('button', { name: 'List view' });
  await listButton.click();

  await expect(page).toHaveURL(/view=list/);
  await page.waitForLoadState('networkidle');
  await expect(results.locator('.module-container.list')).toBeVisible();
  await expect(results.locator('.row')).not.toHaveCount(0);
  await expect(results.locator('.module-card')).toHaveCount(0);

  const filterPanel = page.locator('.modules-search-filters');
  await filterPanel.getByRole('button', { name: 'Weather' }).click();
  await expect(page).toHaveURL(/category=weather/);
  await expect(page).toHaveURL(/view=list/);
  await expect(results.locator('.module-container.list')).toBeVisible();
  await expect(results.locator('.module-card')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'List view' })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
});

test('modules search hides pagination when results fit on one page', async ({ page }) => {
  await page.goto('/modules/search?category=weather');
  await page.waitForLoadState('networkidle');

  const results = page.locator('.results');
  await expect(results).toBeVisible();
  await expect(page.locator('.pagination')).toHaveCount(0);
});
