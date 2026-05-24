import { type CSSProperties, useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ScanMeta } from "./api/scan";
import { aggregate, buildHierarchy, nodeAtPath, type TreeNode } from "./data/hierarchy";
import type { Category, Entry } from "./data/types";
import { useDrillOutKey } from "./hooks/useDrillOutKey";
import { useScanPolling } from "./hooks/useScanPolling";
import { DetailPanel } from "./panels/DetailPanel/DetailPanel";
import { Legend } from "./panels/Legend";
import { StatsRow } from "./panels/StatsRow";
import { Tooltip } from "./panels/Tooltip";
import { ResizeHandle } from "./shell/ResizeHandle";
import { ErrorView, ScanningView } from "./shell/ScanningView";
import { Sidebar, SidebarSection } from "./shell/Sidebar";
import { Topbar } from "./shell/Topbar";
import { useAppState } from "./state/appState";
import {
  type ColumnWidths,
  clampColumn,
  loadStoredColumnWidths,
  persistColumnWidths,
} from "./state/columnWidths";
import { persistTheme } from "./state/theme";
import { ListView } from "./views/ListView";
import { Sunburst } from "./views/Sunburst";
import { Treemap } from "./views/Treemap/Treemap";
import { ViewTabs } from "./views/ViewTabs";

const TREEMAP_PADDING = 0.1;
const TREEMAP_RADIUS = 1;

export function App() {
  const { scan, rescan } = useScanPolling();

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
      scanRoot={scan.scan_root}
      onRescan={rescan}
    />
  );
}

interface LoadedProps {
  data: Entry;
  scannedInMs: number;
  meta: ScanMeta;
  scanRoot: string;
  onRescan: () => void;
}

function Loaded({ data, meta, scanRoot, onRescan }: LoadedProps) {
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

  const agg = useMemo(() => aggregate(root, meta.stale_days), [root, meta.stale_days]);
  const itemCount = useMemo(() => root.descendants().length, [root]);

  // Locate the selected node within the current root by name path.
  const selectedNode = useMemo(() => {
    if (!state.selectedPath) return null;
    const target = state.selectedPath;
    return (
      root.descendants().find((n) => {
        const path = n
          .ancestors()
          .reverse()
          .map((a) => a.data.name);
        return path.length === target.length && path.every((p, i) => p === target[i]);
      }) ?? null
    );
  }, [root, state.selectedPath]);

  const detailNode = selectedNode ?? root;

  const drillOut = useCallback(() => {
    dispatch({ type: "navigateTo", path: state.rootPath.slice(0, -1) });
  }, [dispatch, state.rootPath]);
  useDrillOutKey(state.rootPath, drillOut);

  function handleSelect(node: TreeNode) {
    const path = node
      .ancestors()
      .reverse()
      .map((n) => n.data.name);
    dispatch({ type: "select", path });
  }

  function handleDrillIn(node: TreeNode) {
    if (!node.children || node.children.length === 0) return;
    // Path from data root, excluding the synthetic root name.
    const fullPath = node
      .ancestors()
      .reverse()
      .map((n) => n.data.name);
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
        searchQuery={state.searchQuery}
        onSearchChange={(q) => dispatch({ type: "setSearch", query: q })}
        onToggleTheme={() => dispatch({ type: "toggleTheme" })}
        onRescan={onRescan}
      />

      <div
        className="main"
        style={
          {
            "--left-col": `${columns.left}px`,
            "--right-col": `${columns.right}px`,
          } as CSSProperties
        }
      >
        <DetailPanel
          node={detailNode}
          total={agg.total}
          rootPath={state.rootPath}
          rootName={state.data.name}
          scanRoot={scanRoot}
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
              searchQuery={state.searchQuery}
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
              searchQuery={state.searchQuery}
              rootPathLength={state.rootPath.length}
              maxDepth={state.depthByView.sunburst}
              onSelect={handleSelect}
              onDrillIn={handleDrillIn}
              onUp={drillOut}
              onHover={(node, cursor) => setHover({ node, cursor })}
            />
          ) : (
            <ListView
              root={root}
              selected={selectedNode}
              filterCategories={state.filterCategories}
              searchQuery={state.searchQuery}
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
              onToggle={(category: Category, solo: boolean, visible: ReadonlySet<Category>) =>
                dispatch({ type: "toggleCategory", category, solo, visible })
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
