import type { ViewMode } from "../data/types";
import { MAX_DEPTH_BY_VIEW, MIN_DEPTH } from "../state/appState";
import styles from "./ViewTabs.module.css";

interface ViewTabsProps {
  view: ViewMode;
  itemCount: number;
  depth: number;
  onChange: (view: ViewMode) => void;
  onDepthChange: (depth: number) => void;
}

const TABS: ReadonlyArray<{ value: ViewMode; label: string; icon: JSX.Element }> = [
  {
    value: "treemap",
    label: "Treemap",
    icon: (
      <svg
        viewBox="0 0 16 16"
        width="12"
        height="12"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      >
        <rect x="1.5" y="1.5" width="9" height="6" />
        <rect x="11" y="1.5" width="3.5" height="3" />
        <rect x="11" y="5" width="3.5" height="2.5" />
        <rect x="1.5" y="8" width="6" height="6.5" />
        <rect x="8" y="8" width="6.5" height="6.5" />
      </svg>
    ),
  },
  {
    value: "sunburst",
    label: "Sunburst",
    icon: (
      <svg
        viewBox="0 0 16 16"
        width="12"
        height="12"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      >
        <circle cx="8" cy="8" r="6.5" />
        <circle cx="8" cy="8" r="3.5" />
        <line x1="8" y1="1.5" x2="8" y2="14.5" />
        <line x1="1.5" y1="8" x2="14.5" y2="8" />
      </svg>
    ),
  },
  {
    value: "list",
    label: "List",
    icon: (
      <svg
        viewBox="0 0 16 16"
        width="12"
        height="12"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      >
        <line x1="2" y1="4" x2="14" y2="4" />
        <line x1="2" y1="8" x2="14" y2="8" />
        <line x1="2" y1="12" x2="14" y2="12" />
      </svg>
    ),
  },
];

export function ViewTabs({ view, itemCount, depth, onChange, onDepthChange }: ViewTabsProps) {
  // Depth doesn't apply to the flat list view.
  const showDepth = view !== "list";
  return (
    <div className={styles.viewTabsBar}>
      <div className={styles.seg} role="tablist" aria-label="View mode">
        {TABS.map((t) => (
          <button
            type="button"
            key={t.value}
            className={styles.segBtn}
            role="tab"
            aria-pressed={view === t.value}
            onClick={() => onChange(t.value)}
          >
            {t.icon}
            {t.label}
          </button>
        ))}
      </div>
      <div className={styles.viewTabsMeta}>
        {showDepth ? (
          <label className={styles.depthControl} title="Levels of nesting to render">
            <span className={styles.depthControlLabel}>depth</span>
            <input
              type="range"
              min={MIN_DEPTH}
              max={MAX_DEPTH_BY_VIEW[view]}
              step={1}
              value={depth}
              onChange={(e) => onDepthChange(Number(e.target.value))}
            />
            <span className={`${styles.depthControlValue} mono tabular`}>{depth}</span>
          </label>
        ) : null}
        <span className="mono tabular">{itemCount.toLocaleString()} items</span>
      </div>
    </div>
  );
}
