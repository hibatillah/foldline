const DEFAULT_INGEST_HOST = "https://us.i.posthog.com";

/** The PostHog ingestion host from the environment (US cloud when unset). */
export const ingestHost = process.env.NEXT_PUBLIC_POSTHOG_HOST || DEFAULT_INGEST_HOST;

/** The host that serves PostHog's static scripts, for the same region as the ingestion host. */
export const assetsHost = ingestHost.replace(".i.posthog.com", "-assets.i.posthog.com");

/** The PostHog app host (toolbar and heatmap links), for the same region as the ingestion host. */
export const appHost = ingestHost.replace(".i.posthog.com", ".posthog.com");
