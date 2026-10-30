export type Action =
  | {
      type: "redirect";
      status: number;
      target_url: string;
      path_handling: "replace" | "append";
      query_policy: "drop" | "preserve" | "merge";
    }
  | {
      type: "reverse_proxy";
      upstream_id: string;
      rewrite: string | null;
      preserve_host: boolean;
    };
export type Condition = {
  type: string;
  conditions?: Condition[];
  condition?: Condition;
  [key: string]: unknown;
};
export interface Rule {
  id: string;
  name: string;
  priority: number;
  enabled: boolean;
  condition: Condition;
  action: Action;
  condition_schema_version: number;
}
export interface Mapping {
  id: string;
  mapping_key: string;
  name: string;
  description: string;
  enabled: boolean;
  priority: number;
  source_matcher: {
    hosts: string[];
    path_type: "exact" | "prefix" | "template";
    path_value: string;
    methods: string[];
  };
  default_action: Action;
  rules: Rule[];
  created_at?: string;
  updated_at?: string;
}
export interface Page<T> {
  items: T[];
  total: number;
  offset: number;
}
export interface Entity {
  id: string;
  [key: string]: unknown;
}
export interface Active {
  revision: number;
  draft_generation: number;
  configuration: {
    mappings: Mapping[];
    upstreams: Entity[];
    domains: Entity[];
  };
}
export interface CacheStatus {
  mode: "memory" | "memory_redis";
  connected: boolean;
  active_revision: number;
  database_revision: number;
  last_sync: string | null;
  error: string | null;
}
export interface ImportJob extends Entity {
  status: string;
  base_revision: number;
  conflict_policy: string;
  file_name: string;
  error_count: number;
  total_records: number;
}
