import type { Category } from "./types";

/**
 * Two-tier category model:
 *
 * - `core` categories (cache / build / log / media / vcs / ide / other) are
 *   universal. The legend reserves their color and label even when nothing
 *   matched, so the visual vocabulary stays stable across scans.
 * - `extended` categories (archive / installer / vm_image / model_cache /
 *   backup) are niche. The legend hides them when no entry of that
 *   category is present in the scan, so they don't add noise to a typical
 *   project root but pop into view when relevant (e.g. a 5 GB
 *   `model_cache` block on an AI dev machine).
 */
export type Tier = "core" | "extended";

export interface CategoryMeta {
  key: Category;
  label: string;
  /** One-line description of what kinds of files fall into this category. */
  desc: string;
  tier: Tier;
}

export const CATEGORIES: CategoryMeta[] = [
  // ----- Core -----
  { key: "cache", label: "Cache", desc: "Package and tool caches", tier: "core" },
  { key: "build", label: "Build output", desc: "Build artifacts", tier: "core" },
  { key: "log", label: "Logs", desc: "Log files", tier: "core" },
  { key: "media", label: "Media", desc: "Images, video, audio", tier: "core" },
  { key: "vcs", label: "Version control", desc: ".git, .svn", tier: "core" },
  { key: "ide", label: "IDE config", desc: ".vscode, .idea", tier: "core" },
  { key: "other", label: "Other", desc: "Everything else", tier: "core" },
  // ----- Extended -----
  { key: "archive", label: "Archive", desc: ".zip, .tar.gz, .7z", tier: "extended" },
  { key: "installer", label: "Installer", desc: ".dmg, .exe, .deb", tier: "extended" },
  { key: "vm_image", label: "VM image", desc: ".vdi, .vmdk, .qcow2", tier: "extended" },
  {
    key: "model_cache",
    label: "AI model cache",
    desc: ".ollama, .lmstudio, .huggingface",
    tier: "extended",
  },
  { key: "backup", label: "Backup", desc: "Time Machine, .bak", tier: "extended" },
];

export const ALL_CATEGORIES: ReadonlySet<Category> = new Set(CATEGORIES.map((c) => c.key));

/** Bright fills that need dark text labels in the treemap. */
export const LIGHT_FILL_CATEGORIES: ReadonlySet<Category> = new Set([
  "cache",
  "media",
  "installer",
]);

export function categoryMeta(key: Category): CategoryMeta {
  return CATEGORIES.find((c) => c.key === key) ?? CATEGORIES[CATEGORIES.length - 1]!;
}

/** CSS variable name for a category color, e.g. categoryVar("cache") = "var(--cat-cache)". */
export function categoryVar(key: Category): string {
  return `var(--cat-${key})`;
}
