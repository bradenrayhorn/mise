import { z } from 'zod';

export const schema = z.object({
  name: z.string().trim().min(1, 'Title is required.'),
});

export type TagSchema = typeof schema;
