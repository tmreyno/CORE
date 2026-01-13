// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TreePanel - File tree panel for UFED/AD1/Archive containers
 * 
 * Uses standardized TreeRow component from tree/ for consistent styling.
 */

import { For, Show, createSignal, createMemo } from "solid-js";
import type { ContainerInfo, UfedAssociatedFile } from "../types";
import { TreeRow } from "./tree";
import { HiOutlineFolder } from "./icons";

export interface TreeNode {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  type?: string;
  hash?: string | null;
  children?: TreeNode[];
}

interface TreePanelProps {
  info: ContainerInfo | undefined;
  /** Optional: callback when an entry is selected */
  onSelectEntry?: (node: TreeNode) => void;
  /** Optional: currently selected path */
  selectedPath?: string | null;
}

// Build a tree structure from associated files
function buildTreeFromAssociatedFiles(files: UfedAssociatedFile[], parentFolder?: string): TreeNode[] {
  // Group files by their parent path
  const root: TreeNode = {
    name: parentFolder || "Extraction",
    path: "/",
    isDir: true,
    size: 0,
    children: [],
  };
  
  // Sort files - directories first (indicated by ../), then by name
  const sortedFiles = [...files].sort((a, b) => {
    const aIsParent = a.filename.startsWith("../");
    const bIsParent = b.filename.startsWith("../");
    if (aIsParent && !bIsParent) return 1;
    if (!aIsParent && bIsParent) return -1;
    return a.filename.localeCompare(b.filename);
  });
  
  for (const file of sortedFiles) {
    // Handle files in parent directory (marked with ../)
    if (file.filename.startsWith("../")) {
      const fileName = file.filename.substring(3);
      root.children!.push({
        name: `../${fileName}`,
        path: file.filename,
        isDir: false,
        size: file.size,
        type: file.file_type,
        hash: file.stored_hash,
      });
    } else {
      root.children!.push({
        name: file.filename,
        path: file.filename,
        isDir: false,
        size: file.size,
        type: file.file_type,
        hash: file.stored_hash,
      });
    }
    root.size += file.size;
  }
  
  return root.children!.length > 0 ? [root] : [];
}

/**
 * TreeNodeItem - Internal component using standardized TreeRow
 * 
 * Wraps TreeRow to provide recursive tree rendering.
 */
function TreeNodeItem(props: { 
  node: TreeNode; 
  depth: number;
  expanded: boolean;
  selectedPath: string | null;
  onToggle: (path: string) => void;
  onSelect?: (node: TreeNode) => void;
}) {
  const isSelected = () => props.selectedPath === props.node.path;
  const hasChildren = () => props.node.isDir && (props.node.children?.length ?? 0) > 0;
  
  const handleClick = () => {
    if (props.node.isDir && props.node.children) {
      props.onToggle(props.node.path);
    }
    props.onSelect?.(props.node);
  };
  
  return (
    <>
      <TreeRow
        name={props.node.name}
        path={props.node.path}
        isDir={props.node.isDir}
        size={props.node.size}
        depth={props.depth}
        isSelected={isSelected()}
        isExpanded={props.expanded}
        isLoading={false}
        hasChildren={hasChildren()}
        onClick={handleClick}
        onToggle={() => props.onToggle(props.node.path)}
        entryType={props.node.type}
        hash={props.node.hash}
      />
      <Show when={props.node.isDir && props.expanded && props.node.children}>
        <For each={props.node.children}>
          {(child) => (
            <TreeNodeItem 
              node={child} 
              depth={props.depth + 1}
              expanded={true}
              selectedPath={props.selectedPath}
              onToggle={props.onToggle}
              onSelect={props.onSelect}
            />
          )}
        </For>
      </Show>
    </>
  );
}

export function TreePanel(props: TreePanelProps) {
  const [expandedPaths, setExpandedPaths] = createSignal<Set<string>>(new Set(["/"]) );
  const [selectedPath, setSelectedPath] = createSignal<string | null>(props.selectedPath ?? null);
  
  const togglePath = (path: string) => {
    setExpandedPaths(prev => {
      const newSet = new Set(prev);
      if (newSet.has(path)) {
        newSet.delete(path);
      } else {
        newSet.add(path);
      }
      return newSet;
    });
  };
  
  const handleSelect = (node: TreeNode) => {
    setSelectedPath(node.path);
    props.onSelectEntry?.(node);
  };
  
  // Build tree from container info
  const treeData = createMemo(() => {
    if (!props.info) return [];
    
    // UFED: use associated_files
    if (props.info.ufed?.associated_files) {
      return buildTreeFromAssociatedFiles(
        props.info.ufed.associated_files,
        props.info.ufed.parent_folder || props.info.ufed.device_hint || undefined
      );
    }
    
    // AD1: use tree entries if available
    if (props.info.ad1?.tree) {
      const entries = props.info.ad1.tree;
      const root: TreeNode = {
        name: props.info.ad1.logical.data_source_name || "AD1 Contents",
        path: "/",
        isDir: true,
        size: 0,
        children: [],
      };
      
      // Build tree from flat entries
      for (const entry of entries.slice(0, 50)) { // Limit to first 50 for performance
        root.children!.push({
          name: entry.path.split(/[/\\]/).pop() || entry.path,
          path: entry.path,
          isDir: entry.is_dir,
          size: entry.size,
        });
        root.size += entry.size;
      }
      
      if (entries.length > 50) {
        root.children!.push({
          name: `... and ${entries.length - 50} more items`,
          path: "_more_",
          isDir: false,
          size: 0,
        });
      }
      
      return root.children!.length > 0 ? [root] : [];
    }
    
    // Archive: use entry count hint
    if (props.info.archive) {
      const arch = props.info.archive;
      const root: TreeNode = {
        name: `${arch.format} Archive`,
        path: "/",
        isDir: true,
        size: arch.total_size,
        children: [{
          name: `${arch.entry_count} entries (tree not loaded)`,
          path: "_entries_",
          isDir: false,
          size: 0,
        }],
      };
      return [root];
    }
    
    return [];
  });
  
  const hasTree = () => treeData().length > 0;
  
  return (
    <aside class="flex flex-col bg-zinc-900/50 border-l border-zinc-700 w-64 min-w-[200px]" role="complementary" aria-label="File tree navigation">
      <div class="flex items-center justify-between px-2 py-1 border-b border-zinc-700/50">
        <span class="flex items-center gap-1 text-[10px] font-semibold text-zinc-300">
          <HiOutlineFolder class="w-2.5 h-2.5" /> Files
        </span>
        <Show when={hasTree()}>
          <span class="text-[10px] text-zinc-500">{treeData()[0]?.children?.length || 0}</span>
        </Show>
      </div>
      
      <div class="flex-1 overflow-y-auto" role="tree" aria-label="File tree">
        <Show 
          when={hasTree()}
          fallback={
            <div class="flex flex-col items-center justify-center h-full text-center p-4 text-zinc-500" role="status">
              <span class="text-sm">No file tree available</span>
              <span class="text-xs text-zinc-600 mt-1">Select a UFED, AD1, or archive file</span>
            </div>
          }
        >
          <For each={treeData()}>
            {(node) => (
              <TreeNodeItem 
                node={node} 
                depth={0}
                expanded={expandedPaths().has(node.path)}
                selectedPath={selectedPath()}
                onToggle={togglePath}
                onSelect={handleSelect}
              />
            )}
          </For>
        </Show>
      </div>
    </aside>
  );
}
