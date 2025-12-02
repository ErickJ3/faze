interface PaginationProps {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
  pageSize: number;
  onPageSizeChange: (size: number) => void;
  totalItems: number;
}

const PAGE_SIZES = [10, 25, 50, 100];

export function Pagination({
  currentPage,
  totalPages,
  onPageChange,
  pageSize,
  onPageSizeChange,
  totalItems,
}: PaginationProps) {
  const startItem = (currentPage - 1) * pageSize + 1;
  const endItem = Math.min(currentPage * pageSize, totalItems);

  return (
    <div className="flex items-center justify-between border-t border-border pt-3 mt-4">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <span className="text-xs text-foreground/50">Rows per page:</span>
          <select
            value={pageSize}
            onChange={(e) => onPageSizeChange(Number(e.target.value))}
            className="text-xs bg-background border border-border px-2 py-1 focus:outline-none focus:ring-1 focus:ring-ring"
          >
            {PAGE_SIZES.map((size) => (
              <option key={size} value={size}>
                {size}
              </option>
            ))}
          </select>
        </div>

        <span className="text-xs text-foreground/50">
          {startItem}-{endItem} of {totalItems}
        </span>
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={() => onPageChange(1)}
          disabled={currentPage === 1}
          className="text-xs px-2 py-1 border border-border disabled:opacity-50 disabled:cursor-not-allowed hover:bg-card/50"
        >
          First
        </button>
        <button
          onClick={() => onPageChange(currentPage - 1)}
          disabled={currentPage === 1}
          className="text-xs px-2 py-1 border border-border disabled:opacity-50 disabled:cursor-not-allowed hover:bg-card/50"
        >
          Previous
        </button>
        <span className="text-xs px-2 text-foreground/70">
          Page {currentPage} of {totalPages}
        </span>
        <button
          onClick={() => onPageChange(currentPage + 1)}
          disabled={currentPage === totalPages}
          className="text-xs px-2 py-1 border border-border disabled:opacity-50 disabled:cursor-not-allowed hover:bg-card/50"
        >
          Next
        </button>
        <button
          onClick={() => onPageChange(totalPages)}
          disabled={currentPage === totalPages}
          className="text-xs px-2 py-1 border border-border disabled:opacity-50 disabled:cursor-not-allowed hover:bg-card/50"
        >
          Last
        </button>
      </div>
    </div>
  );
}
