import { EntityManager } from "../components/forms/EntityManager";
import { domains } from "../api/domains";
export default function DomainsPage() {
  return (
    <EntityManager
      name="Domains"
      api={domains}
      defaults={() => ({
        id: crypto.randomUUID(),
        hostname: "",
        enabled: true,
      })}
    />
  );
}
