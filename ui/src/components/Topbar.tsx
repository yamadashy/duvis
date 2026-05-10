import { humanSize } from "../lib/format";
import "./Topbar.css";

interface TopbarProps {
  rootName: string;
  rootSize: number;
  theme: "dark" | "light";
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onToggleTheme: () => void;
  onRescan: () => void;
}

export function Topbar({
  rootName,
  rootSize,
  theme,
  searchQuery,
  onSearchChange,
  onToggleTheme,
  onRescan,
}: TopbarProps) {
  return (
    <div className="topbar">
      <div className="brand">
        <div className="brand-mark" aria-hidden="true">
          <svg viewBox="0 0 16 16" width="14" height="14" fill="none">
            <rect x="2" y="2" width="7" height="7" fill="rgba(255,255,255,.95)" rx="1" />
            <rect x="10" y="2" width="4" height="4" fill="rgba(255,255,255,.7)" rx="1" />
            <rect x="10" y="7" width="4" height="2" fill="rgba(255,255,255,.5)" rx="1" />
            <rect x="2" y="10" width="3" height="4" fill="rgba(255,255,255,.6)" rx="1" />
            <rect x="6" y="10" width="8" height="4" fill="rgba(255,255,255,.85)" rx="1" />
          </svg>
        </div>
        <span className="brand-name">duvis</span>
      </div>

      <div className="topbar-meta">
        <span className="tm-path" title="Scanned root">
          <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M1 5l5-3.5L11 5v5.5a.5.5 0 0 1-.5.5H1.5a.5.5 0 0 1-.5-.5z" />
          </svg>
          {rootName}
        </span>
        <span className="tm-sep" aria-hidden="true" />
        <span className="tm-stat">
          <span className="tm-stat-key">total</span>
          {humanSize(rootSize)}
        </span>
      </div>

      <div className="topbar-search">
        <svg
          className="topbar-search-icon"
          viewBox="0 0 12 12"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          aria-hidden="true"
        >
          <circle cx="5" cy="5" r="3.2" />
          <path d="M7.5 7.5L10 10" strokeLinecap="round" />
        </svg>
        <input
          type="search"
          className="topbar-search-input"
          placeholder="Search names…"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          aria-label="Search entries by name"
        />
        {searchQuery ? (
          <button
            type="button"
            className="topbar-search-clear"
            onClick={() => onSearchChange("")}
            title="Clear search"
            aria-label="Clear search"
          >
            ×
          </button>
        ) : null}
      </div>

      <div className="toolbar">
        <button
          type="button"
          className="icon-btn"
          onClick={onRescan}
          title="Rescan"
          aria-label="Rescan"
        >
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M2 8a6 6 0 0 1 10.5-4M14 8a6 6 0 0 1-10.5 4" />
            <path d="M11.5 1.5V4h-2.5M4.5 14.5V12H7" />
          </svg>
        </button>
        <button
          type="button"
          className="icon-btn"
          onClick={onToggleTheme}
          title={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
          aria-label="Toggle theme"
        >
          {theme === "dark" ? (
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <circle cx="8" cy="8" r="3" />
              <path d="M8 1v1.5M8 13.5V15M3.05 3.05l1.06 1.06M11.89 11.89l1.06 1.06M1 8h1.5M13.5 8H15M3.05 12.95l1.06-1.06M11.89 4.11l1.06-1.06" />
            </svg>
          ) : (
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M13.5 9.5A6 6 0 0 1 6.5 2.5a6 6 0 1 0 7 7z" />
            </svg>
          )}
        </button>
      </div>
    </div>
  );
}
