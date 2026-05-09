import { Fragment, type MouseEvent } from "react";
import { CATEGORIES, type CategoryMeta, categoryVar } from "../lib/categories";
import { humanSize, pct } from "../lib/format";
import type { Category } from "../lib/types";
import "./Legend.css";

interface LegendProps {
  byCategory: Readonly<Record<string, number>>;
  total: number;
  active: ReadonlySet<Category>;
  onToggle: (category: Category, solo: boolean) => void;
}

export function Legend({ byCategory, total, active, onToggle }: LegendProps) {
  // Core categories are always shown so the legend's color vocabulary stays
  // stable across scans. Extended categories are only shown when at least
  // one entry of that category is present, so they don't add noise to a
  // typical project root but pop into view when relevant.
  const core = CATEGORIES.filter((c) => c.tier === "core");
  const extended = CATEGORIES.filter(
    (c) => c.tier === "extended" && (byCategory[c.key] ?? 0) > 0,
  );

  function row(c: CategoryMeta) {
    const isActive = active.has(c.key);
    const size = byCategory[c.key] ?? 0;
    return (
      <button
        type="button"
        key={c.key}
        className="legend-row"
        data-active={isActive}
        onClick={(e: MouseEvent) => onToggle(c.key, e.shiftKey)}
        title={`${c.label} — ${c.desc}`}
      >
        <span className="legend-swatch" style={{ background: categoryVar(c.key) }} />
        <span className="legend-label">{c.label}</span>
        <span className="legend-size mono tabular">{humanSize(size)}</span>
        <span className="legend-pct mono" style={{ fontSize: 10, color: "var(--fg-dim)" }}>
          {pct(size, total)}
        </span>
        <div className="legend-bar">
          <div
            className="legend-bar-fill"
            style={{ width: pct(size, total), background: categoryVar(c.key) }}
          />
        </div>
      </button>
    );
  }

  return (
    <div className="legend">
      {core.map(row)}
      {extended.length > 0 ? (
        <Fragment>
          <div className="legend-divider" aria-hidden="true">
            Extended
          </div>
          {extended.map(row)}
        </Fragment>
      ) : null}
    </div>
  );
}
