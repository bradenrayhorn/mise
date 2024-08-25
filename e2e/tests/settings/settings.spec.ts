import { expect } from '@playwright/test';
import { test } from '@tests/fixtures';

test('can change theme', async ({ login: _, page }) => {
  await page.goto('/settings');

  await expect(page.getByText('Settings', { exact: true })).toBeVisible();

  const body = await page.waitForSelector('body');
  const originalColor = await body.evaluate((el) =>
    window.getComputedStyle(el).getPropertyValue('background-color'),
  );

  await page.getByRole('button', { name: 'Switch to dark mode' }).click();

  const newColor = await body.evaluate((el) =>
    window.getComputedStyle(el).getPropertyValue('background-color'),
  );

  expect(originalColor).not.toBe(newColor);
});

test('can go back to recipes', async ({ login: _, page }) => {
  await page.goto('/settings');

  await page.getByRole('link', { name: 'Back to recipes' }).click();

  await page.waitForURL('/recipes?');
});
