import { expect } from '@playwright/test';
import { createRecipe } from '@tests/apis';
import { test } from '@tests/fixtures';

test('can close page', async ({ login: user, page, request }) => {
  await createRecipe({ user, request, recipe: { title: 'A' } });

  await page.goto('/recipes');
  await page.getByLabel('Search').fill(`${user}`);
  await page.getByLabel('Search').press('Enter');

  await page.getByRole('link', { name: `${user}A` }).click();
  await page.getByRole('link', { name: 'Back to previous page' }).click();

  await expect(page.getByLabel('Search')).toBeVisible();
});
