import { test, expect } from '@playwright/test';

test('install snippet tabs support arrow navigation', async ({ page }) => {
  await page.goto('/');
  await page.waitForLoadState('networkidle');

  const tablist = page.getByRole('tablist', { name: 'Install methods' });
  const shellTab = tablist.getByRole('tab', { name: 'Shell' });
  const aurTab = tablist.getByRole('tab', { name: 'AUR' });
  const sourceTab = tablist.getByRole('tab', { name: 'Source' });
  const shellPanel = page.locator('#install-panel-0');
  const aurPanel = page.locator('#install-panel-1');
  const sourcePanel = page.locator('#install-panel-2');

  await expect(aurTab).toHaveAttribute('aria-selected', 'true');
  await expect(aurPanel).toBeVisible();

  await aurTab.focus();
  await expect(aurTab).toBeFocused();
  await aurTab.press('ArrowRight');

  await expect(sourceTab).toBeFocused();
  await expect(sourceTab).toHaveAttribute('aria-selected', 'true');
  await expect(sourcePanel).toBeVisible();
  await expect(aurPanel).toBeHidden();

  await sourceTab.press('ArrowLeft');

  await expect(aurTab).toBeFocused();
  await expect(aurTab).toHaveAttribute('aria-selected', 'true');
  await expect(aurPanel).toBeVisible();
  await expect(sourcePanel).toBeHidden();

  await aurTab.press('ArrowLeft');

  await expect(shellTab).toBeFocused();
  await expect(shellTab).toHaveAttribute('aria-selected', 'true');
  await expect(shellPanel).toBeVisible();
});

test('modules tabs support arrow navigation', async ({ page }) => {
  await page.goto('/modules');
  await page.waitForLoadState('networkidle');

  const tablist = page.getByRole('tablist', { name: 'Browse navigation' });
  const discoverTab = tablist.getByRole('tab', { name: 'Discover' });
  const searchTab = tablist.getByRole('tab', { name: 'Search' });

  await expect(discoverTab).toHaveAttribute('aria-selected', 'true');
  await discoverTab.focus();
  await expect(discoverTab).toBeFocused();

  await discoverTab.press('ArrowRight');

  await expect(page).toHaveURL(/\/modules\/search/);
  await expect(searchTab).toBeFocused();
  await expect(searchTab).toHaveAttribute('aria-selected', 'true');

  await searchTab.press('ArrowLeft');

  await expect(page).toHaveURL(/\/modules\/?(?:\?.*)?$/);
  await expect(discoverTab).toBeFocused();
  await expect(discoverTab).toHaveAttribute('aria-selected', 'true');
});
