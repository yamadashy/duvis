import { categoryMeta, categoryVar } from "../lib/categories";
import { humanSize, pct, relTime } from "../lib/format";
import type { TreeNode } from "../lib/treemap";
import { isActive } from "../lib/treemap";
import type { Category, SortMode } from "../lib/types";
import "./ListView.css";

interface ListViewProps {
  root: TreeNode;
  selected: TreeNode | null;
  filterCategories: ReadonlySet<Category>;
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
  const { root, selected, filterCategories, sort, onSelect, onDrillIn, onSort, onHover } = props;

  const total = root.value ?? 0;
  const allItems = root.descendants().slice(1);
  const items = allItems.slice(0, MAX_ROWS);
  const truncated = allItems.length > MAX_ROWS;
  const maxVal = items.reduce((m, n) => Math.max(m, n.value ?? 0), 1);

  return (
    <div className="treemap-wrap">
      <div className="list-view">
        <div className="list-head">
          {COLUMNS.map((c) => {
            const active = c.sortAs && c.sortAs === sort;
            return (
              <button
                type="button"
                key={c.key}
                className={`list-head-cell list-col-${c.key}`}
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
          const pathStr = ancestors.length > 0 ? `${ancestors.map((a) => a.data.name).join("/")}/` : "";
          const widthPct = ((n.value ?? 0) / maxVal) * 100;

          return (
            <button
              type="button"
              key={`${i}-${n.data.name}`}
              className="list-row"
              data-dim={!active}
              data-selected={isSel}
              onClick={() => (isDir ? onDrillIn(n) : onSelect(n))}
              onMouseEnter={(e) => onHover(n, e)}
              onMouseMove={(e) => onHover(n, e)}
              onMouseLeave={() => onHover(null, null)}
            >
              <span className="list-name">
                {isDir ? (
                  <svg
                    className="list-name-icon"
                    viewBox="0 0 12 12"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                  >
                    <path d="M1.5 3.5a1 1 0 0 1 1-1h2l1 1h4a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z" />
                  </svg>
                ) : (
                  <svg
                    className="list-name-icon"
                    viewBox="0 0 12 12"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                  >
                    <path d="M3 1.5h4l3 3V10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V2.5a1 1 0 0 1 1-1z" />
                  </svg>
                )}
                {pathStr ? <span className="list-name-path">{pathStr}</span> : null}
                <span className={isDir ? "" : "list-name-leaf"}>
                  {n.data.name}
                  {isDir ? "/" : ""}
                </span>
              </span>
              <span className="list-size">{humanSize(n.value ?? 0)}</span>
              <span className="list-pct">{pct(n.value ?? 0, total)}</span>
              <span className="list-modified">{relTime(days)}</span>
              <span className="list-bar-wrap">
                <span className="list-bar">
                  <span
                    className="list-bar-fill"
                    style={{ width: `${widthPct.toFixed(2)}%`, background: categoryVar(cat) }}
                  />
                </span>
                <span className="list-cat-chip">
                  <span className="list-cat-dot" style={{ background: categoryVar(cat) }} />
                  {meta.label}
                </span>
              </span>
            </button>
          );
        })}

        {truncated ? (
          <div className="list-empty">
            Showing top {MAX_ROWS} of {allItems.length} items. Drill in or filter to narrow.
          </div>
        ) : items.length === 0 ? (
          <div className="list-empty">No items.</div>
        ) : null}
      </div>
    </div>
  );
}
