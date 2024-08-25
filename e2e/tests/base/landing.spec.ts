import { expect } from '@playwright/test';
import { test } from '@tests/fixtures';

test('can login', async ({ page }) => {
  await page.goto('/');

  await page.getByRole('link', { name: 'Log in' }).click();

  await page.getByLabel('Username').fill('charlie');
  await page.getByRole('button', { name: 'Login' }).click();

  await expect(page.getByText('Recipes', { exact: true })).toBeVisible();
});

test('goes straight to recipes if logged in', async ({ login: _, page }) => {
  await page.goto('/');

  await page.waitForURL('/recipes');
  await expect(page.getByText('Recipes', { exact: true })).toBeVisible();
});
