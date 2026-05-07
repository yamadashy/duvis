import type { Category } from "./types";

export interface CategoryMeta {
  key: Category;
  label: string;
  /** One-line description of what kinds of files fall into this category. */
  desc: string;
}

export const CATEGORIES: CategoryMeta[] = [
  { key: "cache", label: "Cache", desc: "Package and tool caches" },
  { key: "build", label: "Build output", desc: "Build artifacts" },
  { key: "log", label: "Logs", desc: "Log files" },
  { key: "media", label: "Media", desc: "Images, video, audio" },
  { key: "vcs", label: "Version control", desc: ".git, .svn" },
  { key: "ide", label: "IDE config", desc: ".vscode, .idea" },
  { key: "other", label: "Other", desc: "Everything else" },
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
