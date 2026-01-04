import { expect, test, type Page } from '@playwright/test';

const viewport = { width: 1280, height: 720 };

async function preparePage(page: Page) {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.addStyleTag({
    content: '*{animation:none !important;transition:none !important;}',
  });
}

async function waitForReady(page: Page) {
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(250);
}

async function gotoFirstModuleDetail(page: Page) {
  await page.goto('/modules');
  await waitForReady(page);
  const link = page.locator('a[href^="/modules/"]').first();
  await expect(link).toBeVisible();
  const href = await link.getAttribute('href');
  if (!href) {
    throw new Error('Module detail link missing');
  }
  const url = new URL(href, page.url()).toString();
  await page.goto(url);
}

test.describe('visual snapshots', () => {
  test.use({ viewport });

  test('landing page', async ({ page }) => {
    await page.goto('/');
    await preparePage(page);
    await waitForReady(page);
    await expect(page).toHaveScreenshot('landing.png', { fullPage: true });
  });

  test('modules discover', async ({ page }) => {
    await page.goto('/modules');
    await preparePage(page);
    await waitForReady(page);
    await expect(page.locator('.browse-discover')).toBeVisible();
    await expect(page).toHaveScreenshot('modules-discover.png', { fullPage: true });
  });

  test('modules search', async ({ page }) => {
    await page.goto('/modules/search?q=clock');
    await preparePage(page);
    await waitForReady(page);
    await expect(page.locator('.modules-search')).toBeVisible();
    await expect(page).toHaveScreenshot('modules-search.png', { fullPage: true });
  });

  test('login', async ({ page }) => {
    await page.goto('/login');
    await preparePage(page);
    await waitForReady(page);
    await expect(page).toHaveScreenshot('login.png', { fullPage: true });
  });

  test('module detail', async ({ page }) => {
    await gotoFirstModuleDetail(page);
    await preparePage(page);
    await waitForReady(page);
    await expect(page).toHaveScreenshot('module-detail.png', { fullPage: true });
  });
});
