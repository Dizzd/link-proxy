import { EntityManager } from "../components/forms/EntityManager";
import { upstreams } from "../api/upstreams";
export default function UpstreamsPage() {
  return (
    <EntityManager
      name="Upstreams"
      api={upstreams}
      defaults={() => ({
        id: crypto.randomUUID(),
        name: "",
        scheme: "https",
        hostname: "",
        port: 443,
        tls_verify: true,
        connect_timeout_ms: 3000,
        read_timeout_ms: 30000,
      })}
    />
  );
}
