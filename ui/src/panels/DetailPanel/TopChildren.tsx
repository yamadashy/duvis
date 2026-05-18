import { categoryVar } from "../../data/categories";
import { humanSize } from "../../data/format";
import type { TreeNode } from "../../data/hierarchy";
import { FileIcon } from "./FileIcon";

interface TopChildrenProps {
  node: TreeNode;
  onSelect: (node: TreeNode) => void;
  onDrillIn: (node: TreeNode) => void;
}

export function TopChildren({ node, onSelect, onDrillIn }: TopChildrenProps) {
  const topChildren = node.children
    ? [...node.children].sort((a, b) => (b.value ?? 0) - (a.value ?? 0)).slice(0, 10)
    : [];

  if (topChildren.length === 0) return null;

  return (
    <div className="detail-section">
      <div className="detail-section-title">Top children</div>
      <div className="detail-children">
        {topChildren.map((c, i) => {
          const cat = c.data.category;
          const isDir = !!c.children && c.children.length > 0;
          return (
            <button
              type="button"
              key={`${i}-${c.data.name}`}
              className="detail-child"
              title={isDir ? "Drill into this folder" : "Inspect file"}
              onClick={() => (isDir ? onDrillIn(c) : onSelect(c))}
            >
              <span className="detail-child-dot" style={{ background: categoryVar(cat) }} />
              <span className="detail-child-name">
                <FileIcon isDir={isDir} />
                {c.data.name}
                {isDir ? "/" : ""}
              </span>
              <span className="detail-child-size mono tabular">
                {humanSize(c.value ?? 0)}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
