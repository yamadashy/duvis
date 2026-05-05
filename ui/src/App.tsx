import type React from "react";
import { useEffect, useMemo, useRef, useState } from "react";
import { DetailPanel } from "./components/DetailPanel";
import { Legend } from "./components/Legend";
import { ListView } from "./components/ListView";
import { ErrorView, ScanningView } from "./components/ScanningView";
import { Sidebar, SidebarSection } from "./components/Sidebar";
import { StatsRow } from "./components/StatsRow";
import { Sunburst } from "./components/Sunburst";
import { Tooltip } from "./components/Tooltip";
import { Topbar } from "./components/Topbar";
import { Treemap } from "./components/Treemap";
import { ViewTabs } from "./components/ViewTabs";
import { ResizeHandle } from "./components/ResizeHandle";
import { fetchScan, requestRescan, type ScanInfo, type ScanMeta } from "./lib/data";
import { aggregate, buildHierarchy, nodeAtPath, type TreeNode } from "./lib/treemap";
import type { Category, Entry } from "./lib/types";
import {
  clampColumn,
  type ColumnWidths,
  loadStoredColumnWidths,
  persistColumnWidths,
  persistTheme,
  useAppState,
} from "./state";

const TREEMAP_PADDING = 0.1;
const TREEMAP_RADIUS = 1;

const POLL_INTERVAL_MS = 500;

export function App() {
  const [scan, setScan] = useState<ScanInfo>({
    status: "scanning",
    elapsed_ms: 0,
    items_scanned: 0,
    scan_root: "",
  });
  // Bumping this restarts the polling effect, used right after a manual rescan.
  const [pollEpoch, setPollEpoch] = useState(0);

  useEffect(() => {
    let cancelled = false;
    let timeoutId: number | undefined;

    async function tick() {
      try {
        const info = await fetchScan();
        if (cancelled) return;
        setScan(info);
        if (info.status === "scanning") {
          timeoutId = window.setTimeout(tick, POLL_INTERVAL_MS);
        }
      } catch (err) {
        if (cancelled) return;
        setScan({
          status: "error",
          message: err instanceof Error ? err.message : String(err),
          scan_root: "",
        });
      }
    }

    tick();
    return () => {
      cancelled = true;
      if (timeoutId) clearTimeout(timeoutId);
    };
  }, [pollEpoch]);

  function rescan() {
    requestRescan().catch(() => {
      // The next poll will surface any error.
    });
    setScan((prev) => ({
      status: "scanning",
      elapsed_ms: 0,
      items_scanned: 0,
      scan_root: prev.scan_root,
    }));
    setPollEpoch((n) => n + 1);
  }

  if (scan.status === "scanning") {
    return (
      <ScanningView
        scanRoot={scan.scan_root}
        elapsedMs={scan.elapsed_ms}
        itemsScanned={scan.items_scanned}
      />
    );
  }
  if (scan.status === "error") {
    return <ErrorView scanRoot={scan.scan_root} message={scan.message} onRescan={rescan} />;
  }
  return (
    <Loaded
      data={scan.tree}
      scannedInMs={scan.scanned_in_ms}
      meta={scan.meta}
      onRescan={rescan}
    />
  );
}

interface LoadedProps {
  data: Entry;
  scannedInMs: number;
  meta: ScanMeta;
  onRescan: () => void;
}

function Loaded({ data, meta, onRescan }: LoadedProps) {
  const deletableCategories = useMemo(
    () => new Set<string>(meta.deletable_categories),
    [meta.deletable_categories],
  );
  const [state, dispatch] = useAppState(data);
  const [hover, setHover] = useState<{
    node: TreeNode | null;
    cursor: { clientX: number; clientY: number } | null;
  }>({ node: null, cursor: null });
  const [columns, setColumns] = useState<ColumnWidths>(() => loadStoredColumnWidths());

  // Persist column widths whenever they settle to a new value.
  useEffect(() => {
    persistColumnWidths(columns);
  }, [columns]);

  function resizeLeft(dx: number) {
    setColumns((prev) => ({ ...prev, left: clampColumn(prev.left + dx) }));
  }
  function resizeRight(dx: number) {
    // Right handle: dragging RIGHT shrinks the right column.
    setColumns((prev) => ({ ...prev, right: clampColumn(prev.right - dx) }));
  }

  // Reset state when fresh data arrives from a rescan with a different shape.
  const lastDataRef = useRef(data);
  useEffect(() => {
    if (lastDataRef.current !== data) {
      lastDataRef.current = data;
      dispatch({ type: "navigateTo", path: [] });
      dispatch({ type: "select", path: null });
    }
  }, [data, dispatch]);

  // Apply theme + accent on document root for token-based styling.
  useEffect(() => {
    document.documentElement.dataset.theme = state.theme;
    document.documentElement.dataset.accent = "indigo";
    persistTheme(state.theme);
  }, [state.theme]);

  // Re-derive the d3 hierarchy whenever data slice or sort changes.
  const root = useMemo(() => {
    const slice = nodeAtPath(state.data, state.rootPath);
    return buildHierarchy(slice, state.sort);
  }, [state.data, state.rootPath, state.sort]);

  const agg = useMemo(
    () => aggregate(root, deletableCategories, meta.stale_days),
    [root, deletableCategories, meta.stale_days],
  );
  const itemCount = useMemo(() => root.descendants().length, [root]);

  // Locate the selected node within the current root by name path.
  const selectedNode = useMemo(() => {
    if (!state.selectedPath) return null;
    const target = state.selectedPath;
    return (
      root.descendants().find((n) => {
        const path = n.ancestors().reverse().map((a) => a.data.name);
        return path.length === target.length && path.every((p, i) => p === target[i]);
      }) ?? null
    );
  }, [root, state.selectedPath]);

  const detailNode = selectedNode ?? root;

  // Drill out via Esc / Backspace.
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.target as HTMLElement | null)?.tagName === "INPUT") return;
      if ((e.key === "Escape" || e.key === "Backspace") && state.rootPath.length > 0) {
        dispatch({ type: "navigateTo", path: state.rootPath.slice(0, -1) });
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [dispatch, state.rootPath]);

  function handleSelect(node: TreeNode) {
    const path = node.ancestors().reverse().map((n) => n.data.name);
    dispatch({ type: "select", path });
  }

  function handleDrillIn(node: TreeNode) {
    if (!node.children || node.children.length === 0) return;
    // Path from data root, excluding the synthetic root name.
    const fullPath = node.ancestors().reverse().map((n) => n.data.name);
    // node.ancestors() top is the current `root` (sliced), so prepend stored rootPath.
    const newRootPath = [...state.rootPath, ...fullPath.slice(1)];
    dispatch({ type: "navigateTo", path: newRootPath });
  }

  return (
    <div className="app">
      <Topbar
        rootName={state.data.name}
        rootSize={state.data.size}
        theme={state.theme}
        onToggleTheme={() => dispatch({ type: "toggleTheme" })}
        onRescan={onRescan}
      />

      <div
        className="main"
        style={
          {
            "--left-col": `${columns.left}px`,
            "--right-col": `${columns.right}px`,
          } as React.CSSProperties
        }
      >
        <DetailPanel
          node={detailNode}
          total={agg.total}
          isRoot={detailNode === root}
          rootPath={state.rootPath}
          rootName={state.data.name}
          deletableCategories={deletableCategories}
          onSelect={handleSelect}
          onDrillIn={handleDrillIn}
          onNavigateTo={(path) => dispatch({ type: "navigateTo", path })}
        />

        <ResizeHandle onDrag={resizeLeft} />

        <div className="stats-wrap">
          <ViewTabs
            view={state.view}
            itemCount={itemCount}
            depth={state.depthByView[state.view]}
            onChange={(v) => dispatch({ type: "setView", view: v })}
            onDepthChange={(d) => dispatch({ type: "setDepth", depth: d })}
          />
          {state.view === "treemap" ? (
            <Treemap
              root={root}
              selected={selectedNode}
              filterCategories={state.filterCategories}
              treemapPadding={TREEMAP_PADDING}
              treemapRadius={TREEMAP_RADIUS}
              maxDepth={state.depthByView.treemap}
              onSelect={handleSelect}
              onDrillIn={handleDrillIn}
              onHover={(node, cursor) => setHover({ node, cursor })}
            />
          ) : state.view === "sunburst" ? (
            <Sunburst
              root={root}
              selected={selectedNode}
              filterCategories={state.filterCategories}
              rootPathLength={state.rootPath.length}
              maxDepth={state.depthByView.sunburst}
              onSelect={handleSelect}
              onDrillIn={handleDrillIn}
              onUp={() => dispatch({ type: "navigateTo", path: state.rootPath.slice(0, -1) })}
              onHover={(node, cursor) => setHover({ node, cursor })}
            />
          ) : (
            <ListView
              root={root}
              selected={selectedNode}
              filterCategories={state.filterCategories}
              sort={state.sort}
              onSelect={handleSelect}
              onDrillIn={handleDrillIn}
              onSort={(s) => dispatch({ type: "setSort", sort: s })}
              onHover={(node, cursor) => setHover({ node, cursor })}
            />
          )}
        </div>

        <ResizeHandle onDrag={resizeRight} />

        <Sidebar>
          <StatsRow agg={agg} itemCount={itemCount} />
          <SidebarSection
            title="Categories"
            action={{ label: "all", onClick: () => dispatch({ type: "resetCategories" }) }}
          >
            <Legend
              byCategory={agg.byCategory}
              total={agg.total}
              active={state.filterCategories}
              onToggle={(category: Category, solo: boolean) =>
                dispatch({ type: "toggleCategory", category, solo })
              }
            />
          </SidebarSection>
        </Sidebar>
      </div>

      <Tooltip
        node={hover.node}
        cursor={hover.cursor}
        total={agg.total}
        rootPath={state.rootPath}
        rootName={state.data.name}
      />
    </div>
  );
}
