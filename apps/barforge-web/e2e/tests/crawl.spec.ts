import { test, expect, Browser, BrowserContext } from '@playwright/test';

const seedRoutes = [
  '/',
  '/modules',
  '/modules/search',
  '/modules/search?q=clock',
  '/modules/search?sort=popular',
  '/modules/search?sort=recent',
  '/modules/search?sort=trending',
  '/modules/search?sort=downloads',
  '/modules/search?sort=alpha',
  '/modules/search?category=weather',
  '/modules/search?page=2',
  '/modules/weather-wttr@barforge',
  '/modules/clock-time@barforge',
  '/modules/cpu-monitor@barforge',
  '/users/barforge',
  '/login',
  '/dashboard',
  '/stars',
  '/collections/ops-essentials',
  '/upload',
  '/admin',
  '/barforge',
  '/terms',
  '/privacy',
  '/settings',
  '/settings/profile',
  '/settings/notifications',
  '/settings/security',
];

const maxButtons = 5;
const maxInputs = 3;
const toastCss = '#__dx-toast { pointer-events: none !important; }';
const crawlConcurrency = 8;
const staticAssetPattern =
  /\.(png|jpe?g|gif|svg|webp|avif|ico|css|js|map|json|woff2?|ttf)$/i;
const ignoredPrefixes = ['/assets/', '/static/', '/build/', '/favicon'];
const maxVariantsPerPath = 3;
const maxRoutes = 200;

const getBasePath = (route: string): string => {
  const queryIndex = route.indexOf('?');
  return queryIndex === -1 ? route : route.slice(0, queryIndex);
};

const getRouteCategory = (route: string): string => {
  const base = getBasePath(route);
  if (base.startsWith('/modules/search')) return 'search';
  if (base.match(/^\/modules\/[^/]+@/)) return 'module-detail';
  if (base.startsWith('/users/')) return 'user-profile';
  if (base.startsWith('/collections/')) return 'collection';
  if (base.startsWith('/settings/')) return 'settings';
  if (base.startsWith('/login')) return 'auth';
  return 'other';
};

const maxPerCategory: Record<string, number> = {
  'search': 8,
  'module-detail': 20,
  'user-profile': 10,
  'collection': 10,
  'settings': 10,
  'auth': 8,
  'other': 100,
};

const isInternalRouteCandidate = (href: string) => {
  if (!href.startsWith('/')) {
    return false;
  }
  if (ignoredPrefixes.some((prefix) => href.startsWith(prefix))) {
    return false;
  }
  if (staticAssetPattern.test(href)) {
    return false;
  }
  if (href.startsWith('/api/')) {
    return false;
  }
  return true;
};

interface RouteResult {
  route: string;
  status: number | 'no-response';
  errors: string[];
  consoleErrors: string[];
  internalLinks: string[];
  externalLinks: string[];
  metrics: {
    buttons: number;
    inputs: number;
    links: number;
    images: number;
    headings: number;
  };
}

async function testRoute(
  context: BrowserContext,
  baseURL: string,
  route: string
): Promise<RouteResult> {
  const page = await context.newPage();
  const errors: string[] = [];
  const consoleErrors: string[] = [];

  page.setDefaultNavigationTimeout(5000);
  page.setDefaultTimeout(3000);

  page.on('pageerror', (err) => {
    errors.push(`pageerror: ${err.message}`);
  });

  page.on('console', (msg) => {
    if (msg.type() === 'error') {
      const text = msg.text();
      if (text.includes('net::ERR_FAILED') || text.includes('net::ERR_ABORTED')) {
        return;
      }
      if (text.includes('fonts.gstatic.com') || text.includes('downloadable font')) {
        return;
      }
      consoleErrors.push(`console.error: ${text}`);
    }
  });

  let status: number | 'no-response' = 'no-response';
  let navigated = false;

  try {
    const response = await page.goto(`${baseURL}${route}`, {
      waitUntil: 'domcontentloaded',
      timeout: 5000,
    });
    status = response ? response.status() : 'no-response';
    navigated = true;
  } catch (err) {
    errors.push(`navigation: ${(err as Error).message}`);
  }

  if (!navigated) {
    await page.close();
    return {
      route,
      status,
      errors,
      consoleErrors,
      internalLinks: [],
      externalLinks: [],
      metrics: { buttons: 0, inputs: 0, links: 0, images: 0, headings: 0 },
    };
  }

  const currentUrl = page.url();
  if (!currentUrl.startsWith(baseURL)) {
    await page.close();
    return {
      route,
      status,
      errors: [],
      consoleErrors: [],
      internalLinks: [],
      externalLinks: [currentUrl],
      metrics: { buttons: 0, inputs: 0, links: 0, images: 0, headings: 0 },
    };
  }

  try {
    await page.addStyleTag({ content: toastCss });
  } catch (err) {
    errors.push(`style-inject: ${(err as Error).message}`);
  }

  const anchors = (await page.evaluate(() =>
    Array.from(
      document.querySelectorAll<HTMLAnchorElement>('a[href]'),
      (anchor) => anchor.getAttribute('href')
    ).filter((href): href is string => Boolean(href))
  ))
    .map((href) => href.split('#')[0])
    .filter((href): href is string => Boolean(href));

  const internalLinks = Array.from(
    new Set(anchors.filter((href) => isInternalRouteCandidate(href)))
  );
  const externalLinks = Array.from(
    new Set(
      anchors.filter(
        (href) => href.startsWith('http://') || href.startsWith('https://')
      )
    )
  );

  const metrics = await page.evaluate(() => ({
    buttons: document.querySelectorAll('button').length,
    inputs: document.querySelectorAll('input, textarea, select').length,
    links: document.querySelectorAll('a[href]').length,
    images: document.querySelectorAll('img').length,
    headings: document.querySelectorAll('h1, h2, h3, h4, h5, h6').length,
  }));

  const buttonLocators = page.locator('button');
  const buttonCount = await buttonLocators.count();
  let clickedButtons = 0;

  for (let i = 0; i < Math.min(buttonCount, maxButtons * 2) && clickedButtons < maxButtons; i++) {
    try {
      const button = buttonLocators.nth(i);
      const isVisible = await button.isVisible({ timeout: 100 }).catch(() => false);
      if (!isVisible) continue;
      const isEnabled = await button.isEnabled({ timeout: 100 }).catch(() => false);
      if (!isEnabled) continue;
      await button.click({ timeout: 200, trial: true, force: true });
      clickedButtons += 1;
    } catch {
      // Ignore button interaction errors
    }
  }

  const inputLocators = page.locator('input:not([type="hidden"]):not([type="submit"]):not([type="button"]), textarea');
  const inputCount = await inputLocators.count();
  let testedInputs = 0;

  for (let i = 0; i < Math.min(inputCount, maxInputs * 2) && testedInputs < maxInputs; i++) {
    try {
      const input = inputLocators.nth(i);
      const isVisible = await input.isVisible({ timeout: 100 }).catch(() => false);
      if (!isVisible) continue;
      const isEnabled = await input.isEnabled({ timeout: 100 }).catch(() => false);
      if (!isEnabled) continue;
      await input.focus({ timeout: 200 });
      testedInputs += 1;
    } catch {
      // Ignore input interaction errors
    }
  }

  await page.close();

  return {
    route,
    status,
    errors,
    consoleErrors,
    internalLinks,
    externalLinks,
    metrics,
  };
}

test.describe('Site Crawl', () => {
  test('crawl internal routes and interactive controls', async ({ browser, baseURL }) => {
    test.setTimeout(45 * 60 * 1000);
    const resolvedBase = baseURL || process.env.PLAYWRIGHT_BASE_URL || 'http://127.0.0.1:8080';
    const startTime = Date.now();
    const log = (msg: string) => console.log(`[${((Date.now() - startTime) / 1000).toFixed(1)}s] ${msg}`);

    log('Creating browser context with resource blocking');
    const context = await browser.newContext();
    await context.route(/\.(png|jpe?g|gif|svg|webp|avif|ico|woff2?|ttf|eot)$/i, (route) => route.abort());
    await context.route(/\.(css)$/i, (route) => route.abort());

    const queue = [...seedRoutes];
    const visited = new Set<string>();
    const discovered = new Set<string>(queue);
    const pathVariantCount = new Map<string, number>();
    const categoryCount = new Map<string, number>();
    const totalMetrics = { buttons: 0, inputs: 0, links: 0, images: 0, headings: 0 };
    const failures: Array<{
      route: string;
      status: number | 'no-response';
      errors: string[];
      consoleErrors: string[];
    }> = [];
    const externalLinksMap = new Map<string, string[]>();

    log('Starting crawl');
    while (queue.length > 0 && visited.size < maxRoutes) {
      const batch: string[] = [];
      while (batch.length < crawlConcurrency && queue.length > 0 && visited.size + batch.length < maxRoutes) {
        const route = queue.shift()!;
        if (visited.has(route)) {
          continue;
        }
        const basePath = getBasePath(route);
        const category = getRouteCategory(route);
        const pathCount = pathVariantCount.get(basePath) ?? 0;
        const catCount = categoryCount.get(category) ?? 0;
        const catMax = maxPerCategory[category] ?? 20;

        if (route !== basePath && pathCount >= maxVariantsPerPath) {
          continue;
        }
        if (catCount >= catMax) {
          continue;
        }

        pathVariantCount.set(basePath, pathCount + 1);
        categoryCount.set(category, catCount + 1);
        visited.add(route);
        batch.push(route);
      }

      if (batch.length === 0) {
        break;
      }

      log(`Batch: ${batch.length} routes (visited=${visited.size}, queued=${queue.length})`);
      const batchStart = Date.now();
      const results = await Promise.all(
        batch.map((route) => testRoute(context, resolvedBase, route))
      );
      log(`Batch done in ${((Date.now() - batchStart) / 1000).toFixed(1)}s`);

      for (const result of results) {
        totalMetrics.buttons += result.metrics.buttons;
        totalMetrics.inputs += result.metrics.inputs;
        totalMetrics.links += result.metrics.links;
        totalMetrics.images += result.metrics.images;
        totalMetrics.headings += result.metrics.headings;

        if (result.externalLinks.length > 0) {
          externalLinksMap.set(result.route, result.externalLinks);
        }

        for (const href of result.internalLinks) {
          if (!discovered.has(href)) {
            discovered.add(href);
            queue.push(href);
          }
        }

        if (result.status === 'no-response') {
          failures.push({
            route: result.route,
            status: result.status,
            errors: result.errors,
            consoleErrors: result.consoleErrors,
          });
          continue;
        }

        if (typeof result.status === 'number' && result.status >= 400) {
          failures.push({
            route: result.route,
            status: result.status,
            errors: result.errors,
            consoleErrors: result.consoleErrors,
          });
          continue;
        }

        if (result.errors.length > 0 || result.consoleErrors.length > 0) {
          failures.push({
            route: result.route,
            status: result.status,
            errors: result.errors,
            consoleErrors: result.consoleErrors,
          });
        }
      }

      if (visited.size % 50 === 0) {
        console.log(
          `crawl progress: visited=${visited.size} queued=${queue.length} discovered=${discovered.size}`
        );
      }
    }

    const categoryBreakdown = Array.from(categoryCount.entries())
      .map(([cat, count]) => `${cat}:${count}`)
      .join(', ');

    const summaryLines = [
      `Visited: ${visited.size}`,
      `Discovered: ${discovered.size}`,
      `Failures: ${failures.length}`,
    ];

    const metricsLine = `Elements: ${totalMetrics.buttons} buttons, ${totalMetrics.inputs} inputs, ${totalMetrics.links} links, ${totalMetrics.headings} headings`;

    test.info().attach('crawl-summary', {
      body: [
        `Routes visited: ${visited.size}`,
        `Routes discovered: ${discovered.size}`,
        `Categories: ${categoryBreakdown}`,
        metricsLine,
        `Failures: ${failures.length}`,
      ].join('\n'),
      contentType: 'text/plain',
    });

    if (externalLinksMap.size > 0) {
      const externalSummary = Array.from(externalLinksMap.entries())
        .map(([route, links]) => `- ${route}: ${links.join(', ')}`)
        .join('\n');
      test.info().attach('external-links', {
        body: externalSummary,
        contentType: 'text/plain',
      });
    }

    if (failures.length > 0) {
      const failureSummary = failures
        .map((failure) => {
          const lines = [`- ${failure.route} (status: ${failure.status})`];
          for (const err of failure.errors) {
            lines.push(`  ${err}`);
          }
          for (const err of failure.consoleErrors) {
            lines.push(`  ${err}`);
          }
          return lines.join('\n');
        })
        .join('\n');
      test.info().attach('crawl-failures', {
        body: `${summaryLines.join('\n')}\n${failureSummary}`,
        contentType: 'text/plain',
      });
    }

    log(`Crawl complete: ${summaryLines.join(' | ')}`);
    await context.close();
    expect(failures, summaryLines.join(' | ')).toEqual([]);
  });
});
