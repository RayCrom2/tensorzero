import { useTensorZeroStatusFetcher } from "~/routes/api/tensorzero/status";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";

/**
 * A component that displays the status of the TensorZero Gateway.
 */
export default function TensorZeroStatusIndicator() {
  const { status, isLoading } = useTensorZeroStatusFetcher();
  const uiVersion = __APP_VERSION__;

  // Extract server version from status if available
  const serverVersion = status?.version || "";

  // Check if versions match (ignoring patch version)
  const versionsMatch = () => {
    if (!serverVersion) return true; // No data yet, don't show warning
    // We can do an exact match for now
    return uiVersion === serverVersion;
  };

  const getStatusColor = () => {
    if (isLoading || status === undefined) return "bg-gray-300"; // Loading or initial state
    if (!status) return "bg-red-500"; // Could not connect (explicit null/failed state)
    if (!versionsMatch()) return "bg-yellow-500"; // Version mismatch
    return "bg-green-500"; // Everything is good
  };

  return (
    <div className="px-3 py-2 text-xs">
      <div className="text-fg-muted flex flex-col gap-1 truncate">
        <div className="flex items-center gap-2">
          <div
            className={`h-2 w-2 rounded-full ${getStatusColor()} mr-1 inline-block`}
          />
          {isLoading
            ? "Checking status..."
            : status === undefined
              ? "Connecting to Gateway..."
              : status
                ? `TensorZero Gateway ${serverVersion}`
                : "Failed to connect to Gateway"}
        </div>
        {status && !versionsMatch() && (
          <div className="ml-5">
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="text-[10px] text-yellow-600">
                  Version mismatch: UI {uiVersion}
                </span>
              </TooltipTrigger>
              <TooltipContent side="right" align="center">
                Please make sure your UI has the same version as the gateway.
                Otherwise you might have compatibility issues.
              </TooltipContent>
            </Tooltip>
          </div>
        )}
      </div>
    </div>
  );
}
