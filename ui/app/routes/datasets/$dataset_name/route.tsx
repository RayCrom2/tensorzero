import {
  getDatasetCounts,
  getDatasetRows,
} from "~/utils/clickhouse/datasets.server";
import type { Route } from "./+types/route";
import DatasetRowTable from "./DatasetRowTable";
import { data, isRouteErrorResponse } from "react-router";
import { useNavigate } from "react-router";
import PageButtons from "~/components/utils/PageButtons";
import DatasetRowSearchBar from "./DatasetRowSearchBar";
import {
  PageHeader,
  PageLayout,
  SectionLayout,
} from "~/components/layout/PageLayout";
import { Toaster } from "~/components/ui/toaster";
import { useToast } from "~/hooks/use-toast";
import { useEffect } from "react";
import { logger } from "~/utils/logger";

export async function loader({ request, params }: Route.LoaderArgs) {
  const { dataset_name } = params;
  const url = new URL(request.url);
  const pageSize = Number(url.searchParams.get("pageSize")) || 15;
  const offset = Number(url.searchParams.get("offset")) || 0;
  const rowsAddedParam = url.searchParams.get("rowsAdded");
  const rowsSkippedParam = url.searchParams.get("rowsSkipped");
  const rowsAdded = rowsAddedParam !== null ? Number(rowsAddedParam) : null;
  const rowsSkipped =
    rowsSkippedParam !== null ? Number(rowsSkippedParam) : null;

  if (pageSize > 100) {
    throw data("Page size cannot exceed 100", { status: 400 });
  }

  const [counts, rows] = await Promise.all([
    getDatasetCounts({}),
    getDatasetRows(dataset_name, pageSize, offset),
  ]);
  const count_info = counts.find(
    (count) => count.dataset_name === dataset_name,
  );
  if (!count_info) {
    throw data("Dataset not found", { status: 404 });
  }
  return { rows, count_info, pageSize, offset, rowsAdded, rowsSkipped };
}

export default function DatasetDetailPage({
  loaderData,
}: Route.ComponentProps) {
  const { rows, count_info, pageSize, offset, rowsAdded, rowsSkipped } =
    loaderData;
  const { toast } = useToast();

  // Use useEffect to show toast only after component mounts
  useEffect(() => {
    if (rowsAdded !== null) {
      toast({
        title: "Dataset Updated",
        description: `Added ${rowsAdded} rows to the dataset. Skipped ${rowsSkipped} duplicate rows.`,
      });
    }
    // TODO: Fix and stop ignoring lint rule
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [rowsAdded, toast]);

  const navigate = useNavigate();
  const handleNextPage = () => {
    const searchParams = new URLSearchParams(window.location.search);
    searchParams.set("offset", String(offset + pageSize));
    navigate(`?${searchParams.toString()}`, { preventScrollReset: true });
  };
  const handlePreviousPage = () => {
    const searchParams = new URLSearchParams(window.location.search);
    searchParams.set("offset", String(offset - pageSize));
    navigate(`?${searchParams.toString()}`, { preventScrollReset: true });
  };

  return (
    <PageLayout>
      <PageHeader heading="Dataset" name={count_info.dataset_name} />
      <SectionLayout>
        <DatasetRowSearchBar dataset_name={count_info.dataset_name} />
        <DatasetRowTable rows={rows} dataset_name={count_info.dataset_name} />
        <PageButtons
          onPreviousPage={handlePreviousPage}
          onNextPage={handleNextPage}
          disablePrevious={offset === 0}
          disableNext={offset + pageSize >= count_info.count}
        />
      </SectionLayout>
      <Toaster />
    </PageLayout>
  );
}

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  logger.error(error);

  if (isRouteErrorResponse(error)) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-4 text-red-500">
        <h1 className="text-2xl font-bold">
          {error.status} {error.statusText}
        </h1>
        <p>{error.data}</p>
      </div>
    );
  } else if (error instanceof Error) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-4 text-red-500">
        <h1 className="text-2xl font-bold">Error</h1>
        <p>{error.message}</p>
      </div>
    );
  } else {
    return (
      <div className="flex h-screen items-center justify-center text-red-500">
        <h1 className="text-2xl font-bold">Unknown Error</h1>
      </div>
    );
  }
}
