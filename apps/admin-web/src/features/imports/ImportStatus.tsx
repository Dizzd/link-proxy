export function ImportStatus({ value }: { value: string }) {
  return (
    <span className="badge" aria-label={`Import status: ${value}`}>
      {value}
    </span>
  );
}
