// Live conformance: drive the generated TypeScript control-plane client against
// a running beaterd and verify typed request/response shapes. Proves API==SDK for TS.

import { Configuration, DatasetsApi, HealthApi, TracesApi } from "../../clients/typescript/src";

async function main(): Promise<number> {
  const base = process.env.BEATER_BASE_URL!;
  const config = new Configuration({ basePath: base });
  const tenantId = "demo";
  const projectId = "demo";

  const health = await new HealthApi(config).healthHealth();
  if (health.ok !== true) {
    console.error(`FAIL health: ${JSON.stringify(health)}`);
    return 1;
  }
  console.log(`  health ok=${health.ok}`);

  await new DatasetsApi(config).datasetsCreateDataset({
    tenantId,
    projectId,
    createDatasetRequest: { name: "conformance-ts" },
  });
  console.log("  createDataset -> ok");

  const page = await new TracesApi(config).tracesListTraces({ tenantId });
  if (!Array.isArray(page.items)) {
    console.error(`FAIL traces.list missing items: ${JSON.stringify(page)}`);
    return 1;
  }
  console.log(`  traces.list items=${page.items.length}`);

  console.log("PASS: typescript generated client round-trips against live API");
  return 0;
}

main()
  .then((c) => process.exit(c))
  .catch((e) => {
    console.error("FAIL:", e?.message ?? e);
    process.exit(1);
  });
