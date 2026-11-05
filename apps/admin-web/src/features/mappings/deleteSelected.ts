export async function deleteSelected(
  ids: readonly string[],
  remove: (id: string) => Promise<unknown>,
) {
  const uniqueIds = [...new Set(ids)];
  if (uniqueIds.length === 0 || uniqueIds.length > 25)
    throw new Error("Select between 1 and 25 mappings on the current page.");
  const deletedIds: string[] = [];
  for (const id of uniqueIds) {
    try {
      await remove(id);
      deletedIds.push(id);
    } catch (error) {
      return { deletedIds, error };
    }
  }
  return { deletedIds, error: null };
}
