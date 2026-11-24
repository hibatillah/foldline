import { useLocale, useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import { installerUrl, RELEASES, releaseUrl } from "@/lib/releases";
import type en from "@/messages/en.json";

import { CtaLink } from "./cta-link";
import { ExternalArrow } from "./external-arrow";
import { PageShell } from "./page-shell";
import { Reveal } from "./reveal";
import { PANEL, TEXTURE, TEXTURE_FILL } from "./textures";
import { WindowsIcon } from "./windows-icon";

type Messages = typeof en;

const ALL_RELEASES = "https://github.com/hibatillah/foldline/releases";

/** The releases page: one entry per version, each with a download card and the changes in plain words. */
export function ReleasesPage() {
	const t = useTranslations();
	const locale = useLocale();
	const entries = t.raw("releases.entries") as Messages["releases"]["entries"];
	const formatDate = new Intl.DateTimeFormat(locale, { dateStyle: "long" });
	const formatSize = (bytes: number) => `${(bytes / 1048576).toFixed(2)} MB`;

	return (
		<PageShell>
			<h1 className="text-[clamp(30px,3vw,40px)] leading-[1.18] font-medium tracking-[-0.03em]">
				{t("releases.title")}
			</h1>
			<p className="mt-4 max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground">
				{t("releases.intro")}
			</p>
			<div className="mt-12 flex flex-col gap-16">
				{RELEASES.map((release, index) => {
					const entry = entries[index];
					return (
						<Reveal key={release.version}>
							<article className="grid grid-cols-[minmax(0,320px)_minmax(0,1fr)] items-start gap-12 max-[900px]:grid-cols-[minmax(0,1fr)] max-[900px]:gap-8">
								<aside
									className={`${PANEL} ${TEXTURE.universtock} ${TEXTURE_FILL} min-h-[320px] flex-col justify-between gap-8 bg-[position:0%_0%] p-6 min-[901px]:sticky min-[901px]:top-24`}
								>
									<div>
										{index === 0 && (
											<p className="m-0 text-[13px] font-medium text-white/80">
												{t("releases.latest")}
											</p>
										)}
										<h2 className="m-0 text-[64px] leading-none font-medium tracking-[-0.04em] text-white tabular-nums">
											{release.version}
										</h2>
										<p className="mt-3 text-[14px] text-white/80">
											{t("releases.released")} {formatDate.format(new Date(release.date))}
										</p>
									</div>
									<div className="flex flex-col gap-4">
										<CtaLink href={installerUrl(release.version)} size="lg" variant="primary">
											<WindowsIcon />
											{t("releases.download")}
										</CtaLink>
										<dl className="m-0 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 rounded-xl bg-white/80 p-4 text-[13px] backdrop-blur-sm">
											<dt className="text-muted-foreground">{t("releases.size")}</dt>
											<dd className="m-0 tabular-nums">{formatSize(release.bytes)}</dd>
											<dt className="text-muted-foreground">{t("releases.checksum")}</dt>
											<dd className="m-0 truncate font-mono text-[12px]" title={release.sha256}>
												{release.sha256}
											</dd>
										</dl>
									</div>
								</aside>
								<div>
									<p className="m-0 text-[clamp(20px,1.8vw,24px)] leading-[1.45] tracking-[-0.01em]">
										{entry.lead}
									</p>
									<div className="mt-10">
										{entry.groups.map((group) => (
											<section
												key={group.h}
												className="grid grid-cols-[120px_minmax(0,1fr)] gap-6 border-t border-border py-8 max-[640px]:grid-cols-[minmax(0,1fr)] max-[640px]:gap-4"
											>
												<h3 className="m-0 text-[15px] font-medium">{group.h}</h3>
												<ul className="m-0 flex list-none flex-col gap-6 p-0">
													{group.items.map((item) => (
														<li key={item.t}>
															<p className="m-0 text-[16px] font-medium">{item.t}</p>
															<p className="mt-1 text-[16px] leading-[1.6] text-muted-foreground">
																{item.d}
															</p>
														</li>
													))}
												</ul>
											</section>
										))}
										<div className="flex flex-wrap items-center gap-x-6 gap-y-3 border-t border-border pt-8">
											<CtaLink href={releaseUrl(release.version)} size="lg" variant="muted">
												{t("releases.changelog")}
												<ExternalArrow />
											</CtaLink>
											<p className="m-0 text-[14px] text-muted-foreground">
												{t("releases.signing")}{" "}
												<Link href="/download" className="text-primary">
													{t("releases.signingLink")}
												</Link>
											</p>
										</div>
									</div>
								</div>
							</article>
						</Reveal>
					);
				})}
			</div>
			<div className="mt-16">
				<CtaLink href={ALL_RELEASES} size="lg" variant="outlineOnly">
					{t("releases.github")}
					<ExternalArrow />
				</CtaLink>
			</div>
		</PageShell>
	);
}
