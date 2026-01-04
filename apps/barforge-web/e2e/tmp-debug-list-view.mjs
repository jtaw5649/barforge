import { chromium } from '@playwright/test';

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1920, height: 1080 } });
await page.goto('http://127.0.0.1:8080/modules');
await page.waitForLoadState('networkidle');
await page.getByRole('tab', { name: 'Search' }).click();
await page.waitForLoadState('networkidle');

const listButtons = await page.locator('button[aria-label="List view"]').elementHandles();
console.log('list buttons', listButtons.length);
for (const [index, handle] of listButtons.entries()) {
  const html = await handle.evaluate((node) => node.outerHTML);
  console.log('button', index, html);
}

const listButton = page.getByRole('button', { name: 'List view' });
await listButton.click();
await page.waitForTimeout(200);

const data = await page.evaluate(() => {
  const button = document.querySelector('button[aria-label="List view"]');
  const container = document.querySelector('.results .module-container');
  const rows = document.querySelectorAll('.results .row').length;
  const cards = document.querySelectorAll('.results .module-card').length;
  return {
    buttonPressed: button?.getAttribute('aria-pressed'),
    containerClass: container?.className,
    rows,
    cards,
  };
});

console.log(data);
await browser.close();
