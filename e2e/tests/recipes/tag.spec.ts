import { expect } from '@playwright/test';
import { test } from '@tests/fixtures';

test('can add new tags while on form', async ({ login: user, page }) => {
  await page.goto('/recipes/new');

  await page.getByRole('button', { name: 'Add Tag' }).click();
  await page.getByRole('menuitem', { name: 'Create Tag' }).click();

  const dialog = page.getByRole('dialog', { name: 'Create Tag' });
  await dialog.getByLabel('Name').fill(`${user}MyTag`);
  await dialog.getByRole('button', { name: 'Save' }).click();

  await page.getByRole('button', { name: 'Add Tag' }).click();
  await expect(page.getByRole('menuitem', { name: `${user}MyTag` })).toBeVisible();
});
