import type { Category } from "./types";

export interface CategoryMeta {
  key: Category;
  label: string;
  /** Tag shown next to the legend label. null when the category isn't reclaimable. */
  tag: "safe" | "warn" | null;
  desc: string;
}

export const CATEGORIES: CategoryMeta[] = [
  { key: "cache", label: "Cache", tag: "safe", desc: "Safely deletable" },
  { key: "build", label: "Build output", tag: "warn", desc: "Rebuildable" },
  { key: "log", label: "Logs", tag: "safe", desc: "Usually deletable" },
  { key: "media", label: "Media", tag: null, desc: "Images, video, audio" },
  { key: "vcs", label: "Version control", tag: null, desc: ".git, .svn" },
  { key: "ide", label: "IDE config", tag: null, desc: ".vscode, .idea" },
  { key: "other", label: "Other", tag: null, desc: "Everything else" },
];

export const ALL_CATEGORIES: ReadonlySet<Category> = new Set(CATEGORIES.map((c) => c.key));

/** Bright fills that need dark text labels in the treemap. */
export const LIGHT_FILL_CATEGORIES: ReadonlySet<Category> = new Set(["cache", "media"]);

export function categoryMeta(key: Category): CategoryMeta {
  return CATEGORIES.find((c) => c.key === key) ?? CATEGORIES[CATEGORIES.length - 1]!;
}

/** CSS variable name for a category color, e.g. categoryVar("cache") = "var(--cat-cache)". */
export function categoryVar(key: Category): string {
  return `var(--cat-${key})`;
}
