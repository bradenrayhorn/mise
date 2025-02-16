import { expect, Page } from '@playwright/test';

export async function addTagFilter({
  page,
  user,
  isMobile,
  tags,
}: {
  page: Page;
  user: string;
  isMobile: boolean;
  tags: string[];
}) {
  if (isMobile) {
    await page.getByRole('button', { name: 'Tag filter' }).click();
    const dialog = page.getByRole('dialog', { name: 'Tags' });
    for (const tag of tags) {
      await dialog.getByRole('checkbox', { name: `${user}${tag}` }).click();
    }
    await dialog.getByRole('button', { name: 'Apply' }).click();
  } else {
    for (const tag of tags) {
      await page.getByRole('button', { name: 'Add Tag' }).click();
      await page.getByRole('menuitem', { name: `${user}${tag}` }).click();
    }
  }
}

export async function removeTagFilter({
  page,
  user,
  isMobile,
  tags,
}: {
  page: Page;
  user: string;
  isMobile: boolean;
  tags: string[];
}) {
  if (isMobile) {
    await page.getByRole('button', { name: 'Tag filter' }).click();
    const dialog = page.getByRole('dialog', { name: 'Tags' });
    for (const tag of tags) {
      await dialog.getByRole('checkbox', { name: `${user}${tag}` }).click();
    }
    await dialog.getByRole('button', { name: 'Apply' }).click();
  } else {
    for (const tag of tags) {
      await page.getByRole('button', { name: `Delete tag ${user}${tag}` }).click();
    }
  }
}

export async function fillRecipeForm({
  page,
  user,
  recipe,
}: {
  page: Page;
  user: string;
  recipe: {
    title: string;
    notes?: string;
    tags: string[];
    image?: string;
    ingredients: { [name: string]: string[] };
    instructions: { [name: string]: string[] };
  };
}) {
  await page.getByLabel('Title').fill(`${user}${recipe.title}`);

  if (recipe.notes) {
    await page.getByLabel('Notes').fill(recipe.notes);
  }

  if (recipe.image) {
    await page.getByLabel('Image').setInputFiles(recipe.image);
  }

  const ingredients = Object.entries(recipe.ingredients);
  for (let i = 0; i < ingredients.length; i++) {
    if (ingredients.length > 1 && i < ingredients.length - 1) {
      await page
        .getByRole('region', { name: 'Ingredients' })
        .getByRole('button', { name: 'Add additional section' })
        .click();
    }

    const [title, items] = ingredients[i];

    if (title) {
      await page.getByLabel(`Ingredient section ${i + 1} title`).fill(title);
    }
    for (let j = 0; j < items.length; j++) {
      await page
        .getByLabel(title ? `${title} ingredient ${j + 1}` : `Ingredient ${j + 1}`)
        .fill(items[j]);
    }
  }

  const instructions = Object.entries(recipe.instructions);
  for (let i = 0; i < instructions.length; i++) {
    if (instructions.length > 1 && i < instructions.length - 1) {
      await page
        .getByRole('region', { name: 'Directions' })
        .getByRole('button', { name: 'Add additional section' })
        .click();
    }

    const [title, items] = instructions[i];

    if (title) {
      await page.getByLabel(`Directions section ${i + 1} title`).fill(title);
    }
    for (let j = 0; j < items.length; j++) {
      await page
        .getByLabel(title ? `${title} direction ${j + 1}` : `Direction ${j + 1}`)
        .fill(items[j]);
    }
  }

  for (const tag of recipe.tags) {
    await page.getByRole('button', { name: 'Add Tag' }).click();
    await page.getByRole('menuitem', { name: `${user}${tag}` }).click();
  }
}

export async function assertRecipeView({
  page,
  user,
  recipe,
}: {
  page: Page;
  user: string;
  recipe: {
    title: string;
    notes?: string;
    tags: string[];
    image: boolean;
    ingredients: { [name: string]: string[] };
    instructions: { [name: string]: string[] };
  };
}) {
  await expect(page.getByRole('heading', { name: `${user}${recipe.title}` })).toBeVisible();

  if (recipe.image) {
    await expect(page.getByRole('img', { name: `${user}${recipe.title}` })).toBeVisible();
  } else {
    await expect(page.getByRole('img', { name: `${user}${recipe.title}` })).not.toBeVisible();
  }

  if (recipe.notes) {
    await expect(page.getByRole('region', { name: 'Notes' })).toHaveText(recipe.notes);
  } else {
    await expect(page.getByRole('region', { name: 'Notes' })).not.toBeVisible();
  }

  for (const [title, items] of Object.entries(recipe.ingredients)) {
    const locators = await page
      .getByRole('region', { name: 'Ingredients' })
      .getByRole('list', { name: title })
      .getByRole('listitem')
      .all();

    expect(locators).toHaveLength(items.length);
    locators.forEach((locator, i) => {
      expect(locator.getByText(items[i])).toBeVisible();
    });
  }
  await expect(page.getByRole('region', { name: 'Ingredients' }).getByRole('list')).toHaveCount(
    Object.entries(recipe.ingredients).length,
  );

  for (const [title, items] of Object.entries(recipe.instructions)) {
    const locators = await page
      .getByRole('region', { name: 'Directions' })
      .getByRole('list', { name: title })
      .getByRole('listitem')
      .all();

    expect(locators).toHaveLength(items.length);
    locators.forEach((locator, i) => {
      expect(locator.getByText(items[i])).toBeVisible();
    });
  }
  await expect(page.getByRole('region', { name: 'Directions' }).getByRole('list')).toHaveCount(
    Object.entries(recipe.instructions).length,
  );

  await expect(page.getByRole('list', { name: 'Tags' }).getByRole('listitem')).toHaveText(
    recipe.tags.map((tag) => `${user}${tag}`),
  );
}

export async function assertRecipeForm({
  page,
  user,
  recipe,
}: {
  page: Page;
  user: string;
  recipe: {
    title: string;
    notes?: string;
    tags: string[];
    image: boolean;
    ingredients: { [name: string]: string[] };
    instructions: { [name: string]: string[] };
  };
}) {
  await expect(page.getByLabel('Title', { exact: true })).toHaveValue(`${user}${recipe.title}`);

  if (recipe.image) {
    await expect(page.getByRole('img', { name: `Uploaded recipe` })).toBeVisible();
  } else {
    await expect(page.getByRole('img', { name: `Uploaded recipe` })).not.toBeVisible();
  }

  if (recipe.notes) {
    await expect(page.getByLabel('Notes')).toHaveValue(recipe.notes);
  } else {
    await expect(page.getByLabel('Notes')).toHaveValue('');
  }

  const ingredients = Object.entries(recipe.ingredients);
  for (let i = 0; i < ingredients.length; i++) {
    const [title, items] = ingredients[i];

    if (title) {
      await expect(page.getByLabel(`Ingredient section ${i + 1} title`)).toHaveValue(title);
    } else {
      expect(page.getByLabel(`Ingredient section ${i + 1} title`)).not.toBeVisible();
    }

    for (let j = 0; j < items.length; j++) {
      await expect(
        page.getByLabel(title ? `${title} ingredient ${j + 1}` : `Ingredient ${j + 1}`),
      ).toHaveValue(items[j]);
    }

    // make sure that was the last ingredient
    await expect(
      page.getByLabel(
        title ? `${title} ingredient ${items.length + 2}` : `Ingredient ${items.length + 2}`,
      ),
    ).not.toBeVisible();
  }

  // make sure that was the last ingredient block
  await expect(page.getByLabel(`Ingredient section ${ingredients.length + 2}`)).not.toBeVisible();

  const instructions = Object.entries(recipe.instructions);
  for (let i = 0; i < instructions.length; i++) {
    const [title, items] = instructions[i];

    if (title) {
      await expect(page.getByLabel(`Directions section ${i + 1} title`)).toHaveValue(title);
    } else {
      expect(page.getByLabel(`Directions section ${i + 1} title`)).not.toBeVisible();
    }

    for (let j = 0; j < items.length; j++) {
      await expect(
        page.getByLabel(title ? `${title} direction ${j + 1}` : `Direction ${j + 1}`),
      ).toHaveValue(items[j]);
    }

    // make sure that was the last instruction
    await expect(
      page.getByLabel(
        title ? `${title} direction ${items.length + 2}` : `Direction ${items.length + 2}`,
      ),
    ).not.toBeVisible();
  }

  // make sure that was the last instruction block
  await expect(page.getByLabel(`Directions section ${instructions.length + 2}`)).not.toBeVisible();

  await expect(page.getByRole('list', { name: 'Tags' }).getByRole('listitem')).toHaveText(
    recipe.tags.map((tag) => `${user}${tag}`),
  );
}
