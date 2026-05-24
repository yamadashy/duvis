import { useMemo } from "react";
import { categoryMeta, categoryVar } from "../data/categories";
import { humanSize, pct, relTime } from "../data/format";
import type { TreeNode } from "../data/hierarchy";
import { isActive, nameMatchesSearch, normalizeSearchQuery } from "../data/search";
import type { Category, SortMode } from "../data/types";
import styles from "./ListView.module.css";

interface ListViewProps {
  root: TreeNode;
  selected: TreeNode | null;
  filterCategories: ReadonlySet<Category>;
  searchQuery: string;
  sort: SortMode;
  onSelect: (node: TreeNode) => void;
  onDrillIn: (node: TreeNode) => void;
  onSort: (sort: SortMode) => void;
  onHover: (node: TreeNode | null, evt: { clientX: number; clientY: number } | null) => void;
}

const MAX_ROWS = 200;

const COLUMNS: ReadonlyArray<{
  key: "name" | "size" | "pct" | "modified" | "bar";
  label: string;
  sortAs?: SortMode;
  align?: "right";
}> = [
  { key: "name", label: "Name", sortAs: "name" },
  { key: "size", label: "Size", sortAs: "size", align: "right" },
  { key: "pct", label: "%", align: "right" },
  { key: "modified", label: "Modified", sortAs: "oldest", align: "right" },
  { key: "bar", label: "Distribution" },
];

export function ListView(props: ListViewProps) {
  const {
    root,
    selected,
    filterCategories,
    searchQuery,
    sort,
    onSelect,
    onDrillIn,
    onSort,
    onHover,
  } = props;

  const total = root.value ?? 0;
  // Normalize the query once per render rather than per-row.
  const loweredQuery = useMemo(() => normalizeSearchQuery(searchQuery), [searchQuery]);
  // List view actively filters on search (rows scroll, so dimming is less
  // useful than in a fixed-area treemap). Category filter still dims so
  // the legend toggles stay consistent across views.
  const allItems = root
    .descendants()
    .slice(1)
    .filter((n) => nameMatchesSearch(n, loweredQuery));
  const items = allItems.slice(0, MAX_ROWS);
  const truncated = allItems.length > MAX_ROWS;
  const maxVal = items.reduce((m, n) => Math.max(m, n.value ?? 0), 1);

  return (
    <div className="view-wrap">
      <div className={styles.listView}>
        <div className={styles.listHead}>
          {COLUMNS.map((c) => {
            const active = c.sortAs && c.sortAs === sort;
            const cls =
              c.align === "right"
                ? `${styles.listHeadCell} ${styles.listColRight}`
                : styles.listHeadCell;
            return (
              <button
                type="button"
                key={c.key}
                className={cls}
                data-active={!!active}
                data-sortable={!!c.sortAs}
                onClick={() => c.sortAs && onSort(c.sortAs)}
              >
                {c.label}
                {active ? " ↓" : ""}
              </button>
            );
          })}
        </div>

        {items.map((n, i) => {
          const cat: Category = n.data.category;
          const meta = categoryMeta(cat);
          const active = isActive(n, filterCategories);
          const isSel = selected ? selected === n : false;
          const days = n.data.modified_days_ago ?? 0;
          const isDir = !!n.children && n.children.length > 0;
          const ancestors = n.ancestors().reverse().slice(1, -1);
          const pathStr =
            ancestors.length > 0 ? `${ancestors.map((a) => a.data.name).join("/")}/` : "";
          const widthPct = ((n.value ?? 0) / maxVal) * 100;

          return (
            <button
              type="button"
              // biome-ignore lint/suspicious/noArrayIndexKey: rows are sorted by size each render, names can collide
              key={`${i}-${n.data.name}`}
              className={styles.listRow}
              data-dim={!active}
              data-selected={isSel}
              onClick={() => (isDir ? onDrillIn(n) : onSelect(n))}
              onMouseEnter={(e) => onHover(n, e)}
              onMouseMove={(e) => onHover(n, e)}
              onMouseLeave={() => onHover(null, null)}
            >
              <span className={styles.listName}>
                {isDir ? (
                  <svg
                    className={styles.listNameIcon}
                    viewBox="0 0 12 12"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    aria-hidden="true"
                  >
                    <path d="M1.5 3.5a1 1 0 0 1 1-1h2l1 1h4a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z" />
                  </svg>
                ) : (
                  <svg
                    className={styles.listNameIcon}
                    viewBox="0 0 12 12"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    aria-hidden="true"
                  >
                    <path d="M3 1.5h4l3 3V10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V2.5a1 1 0 0 1 1-1z" />
                  </svg>
                )}
                {pathStr ? <span className={styles.listNamePath}>{pathStr}</span> : null}
                <span className={isDir ? "" : styles.listNameLeaf}>
                  {n.data.name}
                  {isDir ? "/" : ""}
                </span>
              </span>
              <span className={styles.listSize}>{humanSize(n.value ?? 0)}</span>
              <span className={styles.listPct}>{pct(n.value ?? 0, total)}</span>
              <span className={styles.listModified}>{relTime(days)}</span>
              <span className={styles.listBarWrap}>
                <span className={styles.listBar}>
                  <span
                    className={styles.listBarFill}
                    style={{ width: `${widthPct.toFixed(2)}%`, background: categoryVar(cat) }}
                  />
                </span>
                <span className={styles.listCatChip}>
                  <span className={styles.listCatDot} style={{ background: categoryVar(cat) }} />
                  {meta.label}
                </span>
              </span>
            </button>
          );
        })}

        {truncated ? (
          <div className={styles.listEmpty}>
            Showing top {MAX_ROWS} of {allItems.length} items. Drill in or filter to narrow.
          </div>
        ) : items.length === 0 ? (
          <div className={styles.listEmpty}>
            {searchQuery ? `No entries match "${searchQuery}".` : "No items."}
          </div>
        ) : null}
      </div>
    </div>
  );
}
