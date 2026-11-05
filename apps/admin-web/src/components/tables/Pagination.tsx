export function Pagination({
  offset,
  total,
  pageSize,
  onChange,
  disabled = false,
}: {
  offset: number;
  total: number;
  pageSize: number;
  onChange: (offset: number) => void;
  disabled?: boolean;
}) {
  return (
    <div className="pagination">
      <button
        disabled={disabled || !offset}
        onClick={() => onChange(Math.max(0, offset - pageSize))}
      >
        Previous
      </button>
      <span>
        {total ? offset + 1 : 0}-{Math.min(offset + pageSize, total)} of {total}
      </span>
      <button
        disabled={disabled || offset + pageSize >= total}
        onClick={() => onChange(offset + pageSize)}
      >
        Next
      </button>
    </div>
  );
}
