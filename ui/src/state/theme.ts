export type Theme = "dark" | "light";

const THEME_STORAGE_KEY = "duvis.theme";

export function loadStoredTheme(): Theme {
  try {
    const v = localStorage.getItem(THEME_STORAGE_KEY);
    if (v === "dark" || v === "light") return v;
  } catch {
    // ignore (private mode etc.)
  }
  return "light";
}

export function persistTheme(theme: Theme): void {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // ignore
  }
}
