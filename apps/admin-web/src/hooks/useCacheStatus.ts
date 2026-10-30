import { useQuery } from "@tanstack/react-query";
import { config } from "../api/config";

export function useCacheStatus() {
  return useQuery({
    queryKey: ["cache"],
    queryFn: config.cacheStatus,
    refetchInterval: 5000,
    staleTime: 5000,
    retry: 1,
  });
}
