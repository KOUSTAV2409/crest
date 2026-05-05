/** Normalized slug for `[data-category="…"]` styling (URLs not used). */
export function categorySlug(category: string): string {
  return category.toLowerCase().replace(/\s+/g, '-');
}
