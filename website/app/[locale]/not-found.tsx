import { useTranslations } from "next-intl";

import { CtaLink } from "@/components/landing/cta-link";
import { PageShell } from "@/components/landing/page-shell";
import { StatusPage } from "@/components/landing/status-page";

/** Shown for an unknown address and for any page that calls notFound(); the catch-all route sends unknown addresses here. */
export default function NotFound() {
	const t = useTranslations();

	return (
		<PageShell>
			<StatusPage code={t("notFound.code")} title={t("notFound.a")} text={t("notFound.b")}>
				<CtaLink href="/" size="lg" variant="primary">
					{t("notFound.home")}
				</CtaLink>
			</StatusPage>
		</PageShell>
	);
}
