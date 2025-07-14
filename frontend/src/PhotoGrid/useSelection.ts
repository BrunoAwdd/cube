import { useState } from "react";

export function useSelection<T extends { id: string }>(items: T[]) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  const toggleSelection = (id: string) => {
    setSelectedIds((prev) => {
      const copy = new Set(prev);
      copy.has(id) ? copy.delete(id) : copy.add(id);
      return copy;
    });
  };

  const clearSelection = () => setSelectedIds(new Set());

  const selectAll = () => {
    setSelectedIds(new Set(items.map((i) => i.id)));
  };

  const isSelected = (id: string) => selectedIds.has(id);

  const selectedItems = items.filter((i) => selectedIds.has(i.id));

  return {
    selectedIds,
    selectedItems,
    toggleSelection,
    isSelected,
    clearSelection,
    selectAll,
  };
}
