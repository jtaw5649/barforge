import { test, expect } from '@playwright/test';

test('install snippet copy button toggles and command expands', async ({ page, context, browserName }) => {
  if (browserName !== 'firefox') {
    await context.grantPermissions(['clipboard-write']);
  }
  await page.goto('/');
  await page.waitForLoadState('networkidle');

  const snippet = page.locator('.install-snippet');
  await expect(snippet).toBeVisible();

  const activePanel = snippet.locator('.install-panel:not([hidden])');
  await expect(activePanel).toBeVisible();
  const terminalBody = activePanel.locator('.terminal-body');
  const command = terminalBody.locator('.terminal-cmd');

  await expect(command).not.toHaveClass(/show-full/);
  await terminalBody.click();
  await expect(command).toHaveClass(/show-full/);

  const copyButton = terminalBody.locator('.copy-btn');
  await copyButton.click();

  if (browserName !== 'firefox') {
    await expect(copyButton).toHaveAttribute('aria-label', 'Copied');
    await expect(copyButton).toBeDisabled();
    await expect(copyButton).toContainText('Copied!');
  } else {
    await expect(copyButton).toHaveAttribute('aria-label', /Copy to clipboard|Copied/);
  }
});
