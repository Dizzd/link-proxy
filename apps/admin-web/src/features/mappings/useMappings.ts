import { useQuery } from "@tanstack/react-query";
import { mappings } from "../../api/mappings";
export function useMappings(query: string) {
  return useQuery({
    queryKey: ["mappings", query],
    queryFn: () => mappings.list(query),
  });
}
