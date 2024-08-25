import { expect } from '@playwright/test';
import { createRecipe, createTag } from '@tests/apis';
import { test } from '@tests/fixtures';
import { assertRecipeView, fillRecipeForm, assertRecipeForm } from '@tests/helpers';

test('can close page', async ({ login: user, page, request }) => {
  await createRecipe({ user, request, recipe: { title: 'A' } });

  await page.goto('/recipes');
  await page.getByLabel('Search').fill(`${user}`);
  await page.getByLabel('Search').press('Enter');

  // to view page
  await page.getByRole('link', { name: `${user}A` }).click();

  // to edit page
  await page.getByRole('link', { name: 'Edit' }).click();
  await expect(page.getByRole('heading', { name: 'Edit Recipe' })).toBeVisible();

  // back to view page
  await page.getByRole('link', { name: 'Back to previous page' }).click();
  await expect(page.getByRole('heading', { name: `${user}A` })).toBeVisible();
});

test('can edit recipe to include simple details', async ({ login: user, page, request }) => {
  await createTag({ user, request, name: 'Good' });
  await createTag({ user, request, name: 'Quick' });

  const recipeID = await createRecipe({ user, request, recipe: { title: 'A' } });

  // go to form
  await page.goto(`/recipes/${recipeID}/edit`);

  // fill out form
  await fillRecipeForm({
    user,
    page,
    recipe: {
      title: 'Dinner',
      image: 'fixtures/recipe-image.jpg',
      notes: 'Tastes good!',
      ingredients: { '': ['Apple sauce', 'Corn starch'] },
      instructions: { '': ['Stir the apple sauce and corn starch.', 'Bake at 350.'] },
      tags: ['Good', 'Quick'],
    },
  });

  await page.getByRole('button', { name: 'Save' }).click();

  // verify details
  await assertRecipeView({
    user,
    page,
    recipe: {
      title: 'Dinner',
      image: true,
      notes: 'Tastes good!',
      ingredients: { Ingredients: ['Apple sauce', 'Corn starch'] },
      instructions: { Instructions: ['Stir the apple sauce and corn starch.', 'Bake at 350.'] },
      tags: ['Good', 'Quick'],
    },
  });

  // go back to edit page and check the form
  await page.getByRole('link', { name: 'Edit' }).click();
  await assertRecipeForm({
    user,
    page,
    recipe: {
      title: 'Dinner',
      image: true,
      notes: 'Tastes good!',
      ingredients: { '': ['Apple sauce', 'Corn starch'] },
      instructions: { '': ['Stir the apple sauce and corn starch.', 'Bake at 350.'] },
      tags: ['Good', 'Quick'],
    },
  });
});

test('can edit recipe to include complex details', async ({ login: user, page, request }) => {
  await createTag({ user, request, name: 'Good' });

  const recipeID = await createRecipe({ user, request, recipe: { title: 'A' } });

  // go to form
  await page.goto(`/recipes/${recipeID}/edit`);

  // fill out form
  await fillRecipeForm({
    user,
    page,
    recipe: {
      title: 'Glazed Bread',
      image: 'fixtures/recipe-image.jpg',
      notes: 'Tastes good!',
      ingredients: { Glaze: ['Milk', 'Water'], Bread: ['Flour', 'Eggs'] },
      instructions: { Glaze: ['Stir', 'Heat'], Bread: ['Stir', 'Bake'] },
      tags: ['Good'],
    },
  });

  await page.getByRole('button', { name: 'Save' }).click();

  // verify details
  await assertRecipeView({
    user,
    page,
    recipe: {
      title: 'Glazed Bread',
      image: true,
      notes: 'Tastes good!',
      ingredients: { Glaze: ['Milk', 'Water'], Bread: ['Flour', 'Eggs'] },
      instructions: { Glaze: ['Stir', 'Heat'], Bread: ['Stir', 'Bake'] },
      tags: ['Good'],
    },
  });

  // go back to edit page and check the form
  await page.getByRole('link', { name: 'Edit' }).click();
  await assertRecipeForm({
    user,
    page,
    recipe: {
      title: 'Glazed Bread',
      image: true,
      notes: 'Tastes good!',
      ingredients: { Glaze: ['Milk', 'Water'], Bread: ['Flour', 'Eggs'] },
      instructions: { Glaze: ['Stir', 'Heat'], Bread: ['Stir', 'Bake'] },
      tags: ['Good'],
    },
  });
});

test('can remove details from recipe', async ({ login: user, page, request }) => {
  const tagID = await createTag({ user, request, name: 'Good' });

  const recipeID = await createRecipe({
    user,
    request,
    recipe: {
      title: 'Chicken Soup!',
      notes: 'Lots of detail here.',
      tagIDs: [tagID],
      ingredients: { Glaze: ['Milk', 'Water'], Bread: ['Flour', 'Eggs'] },
      instructions: { Glaze: ['Stir', 'Heat'], Bread: ['Stir', 'Bake'] },
    },
  });

  // go to add form
  await page.goto(`/recipes/${recipeID}/edit`);

  // clear out form
  await page.getByLabel('Title', { exact: true }).fill(`${user}Just a title`);
  await page.getByLabel('Notes').clear();

  await page
    .getByRole('region', { name: 'Ingredients' })
    .getByRole('button', { name: 'Delete Bread ingredients' })
    .click();
  await page.getByLabel('Ingredient 1').clear();
  await page.getByLabel('Ingredient 2').clear();

  await page
    .getByRole('region', { name: 'Instructions' })
    .getByRole('button', { name: 'Delete Bread instructions' })
    .click();
  await page.getByLabel('Instruction 1').clear();
  await page.getByLabel('Instruction 2').clear();

  await page.getByRole('button', { name: `Delete tag ${user}Good` }).click();

  await page.getByRole('button', { name: 'Save' }).click();

  // verify details
  await assertRecipeView({
    user,
    page,
    recipe: {
      title: 'Just a title',
      image: false,
      ingredients: {},
      instructions: {},
      tags: [],
    },
  });

  // go back to edit page and check the form
  await page.getByRole('link', { name: 'Edit' }).click();
  await assertRecipeForm({
    user,
    page,
    recipe: {
      title: 'Just a title',
      image: false,
      ingredients: {},
      instructions: {},
      tags: [],
    },
  });
});
