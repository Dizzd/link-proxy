import { describe, expect, it, vi } from "vitest";
import { deleteSelected } from "./deleteSelected";

describe("deleteSelected", () => {
  it("awaits each deletion before starting the next and deduplicates IDs", async () => {
    let finish!: () => void;
    const remove = vi
      .fn()
      .mockImplementationOnce(
        () =>
          new Promise<void>((resolve) => {
            finish = resolve;
          }),
      )
      .mockResolvedValue(undefined);
    const pending = deleteSelected(["a", "b", "a"], remove);
    expect(remove.mock.calls).toEqual([["a"]]);
    finish();
    expect(await pending).toEqual({ deletedIds: ["a", "b"], error: null });
    expect(remove.mock.calls).toEqual([["a"], ["b"]]);
  });

  it("stops at the first failure and reports only completed deletions", async () => {
    const error = new Error("Session expired");
    const remove = vi
      .fn()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValue(error);
    expect(await deleteSelected(["a", "b", "c"], remove)).toEqual({
      deletedIds: ["a"],
      error,
    });
    expect(remove.mock.calls).toEqual([["a"], ["b"]]);
  });

  it("rejects an empty or oversized selection before deleting anything", async () => {
    const remove = vi.fn();
    await expect(deleteSelected([], remove)).rejects.toThrow(
      "between 1 and 25",
    );
    await expect(
      deleteSelected(
        Array.from({ length: 26 }, (_, i) => String(i)),
        remove,
      ),
    ).rejects.toThrow("between 1 and 25");
    expect(remove).not.toHaveBeenCalled();
  });

  it("accepts a full page of 25 mappings", async () => {
    const ids = Array.from({ length: 25 }, (_, i) => String(i));
    const remove = vi.fn().mockResolvedValue(undefined);
    expect(await deleteSelected(ids, remove)).toEqual({
      deletedIds: ids,
      error: null,
    });
    expect(remove).toHaveBeenCalledTimes(25);
  });
});
