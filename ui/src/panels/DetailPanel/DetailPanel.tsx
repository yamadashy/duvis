import { categoryMeta, categoryVar } from "../../data/categories";
import { humanSize, pct } from "../../data/format";
import type { TreeNode } from "../../data/hierarchy";
import { ActionRow } from "./actions";
import { Breadcrumbs } from "./Breadcrumbs";
import "./DetailPanel.css";
import { MetadataGrid } from "./MetadataGrid";
import { TopChildren } from "./TopChildren";

interface DetailPanelProps {
  node: TreeNode;
  total: number;
  /** Path of names from data root to the current view root (rootPath in App state). */
  rootPath: readonly string[];
  rootName: string;
  /** Absolute filesystem path the scan was rooted at. Used to assemble the
   *  full path for Copy Path. */
  scanRoot: string;
  onDrillIn: (node: TreeNode) => void;
  onSelect: (node: TreeNode) => void;
  onNavigateTo: (path: string[]) => void;
}

export function DetailPanel(props: DetailPanelProps) {
  const { node, total, rootPath, rootName, scanRoot, onDrillIn, onSelect, onNavigateTo } = props;
  const cat = node.data.category;
  const meta = categoryMeta(cat);

  // Full breadcrumb from data root through view root to selected node.
  const inViewSegments = node
    .ancestors()
    .reverse()
    .slice(1)
    .map((a) => a.data.name);
  const fullSegments = [rootName, ...rootPath, ...inViewSegments];
  const actionSegments = [...rootPath, ...inViewSegments];

  return (
    <aside className="detail" aria-label="Selection details">
      <div className="detail-head">
        <Breadcrumbs segments={fullSegments} onNavigateTo={onNavigateTo} />

        <div className="detail-size tabular">
          {humanSize(node.value ?? 0)}
          <span className="detail-size-pct">{pct(node.value ?? 0, total)} of root</span>
        </div>
        <div className="detail-cat-row">
          <span className="detail-cat-chip">
            <span className="detail-cat-chip-dot" style={{ background: categoryVar(cat) }} />
            {meta.label}
          </span>
        </div>
      </div>

      <TopChildren node={node} onSelect={onSelect} onDrillIn={onDrillIn} />

      <MetadataGrid node={node} />

      <div className="detail-section">
        <ActionRow node={node} scanRoot={scanRoot} segments={actionSegments} total={total} />
      </div>
    </aside>
  );
}
