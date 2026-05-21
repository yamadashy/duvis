import { categoryVar } from "../../data/categories";
import { humanSize } from "../../data/format";
import type { TreeNode } from "../../data/hierarchy";
import styles from "./DetailPanel.module.css";
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
    <div className={styles.detailSection}>
      <div className={styles.detailSectionTitle}>Top children</div>
      <div className={styles.detailChildren}>
        {topChildren.map((c, i) => {
          const cat = c.data.category;
          const isDir = c.data.is_dir;
          return (
            <button
              type="button"
              key={`${i}-${c.data.name}`}
              className={styles.detailChild}
              title={isDir ? "Drill into this folder" : "Inspect file"}
              onClick={() => (isDir ? onDrillIn(c) : onSelect(c))}
            >
              <span className={styles.detailChildDot} style={{ background: categoryVar(cat) }} />
              <span className={styles.detailChildName}>
                <FileIcon isDir={isDir} />
                {c.data.name}
                {isDir ? "/" : ""}
              </span>
              <span className={`${styles.detailChildSize} mono tabular`}>
                {humanSize(c.value ?? 0)}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
