import { expect } from '@playwright/test';
import { createRecipe } from '@tests/apis';
import { test } from '@tests/fixtures';

test('can close page', async ({ login: user, page, request }) => {
  await createRecipe({ user, request, recipe: { title: 'A' } });

  await page.goto('/recipes');
  await page.getByRole('textbox', { name: 'Search' }).fill(`${user}`);
  await page.getByRole('textbox', { name: 'Search' }).press('Enter');

  await page.getByRole('link', { name: `${user}A` }).click();
  await page.getByRole('link', { name: 'Back' }).click();

  await expect(page.getByRole('textbox', { name: 'Search' })).toBeVisible();
});
