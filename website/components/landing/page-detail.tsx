import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import {
	CopiedPill,
	DeviceBatteryPill,
	EventPill,
	PrayerPill,
	RainPill,
	ShelfPill,
	TimerDonePill,
	TimerPill,
	WavePill,
} from "@/components/notch/pill";
import { Link } from "@/i18n/navigation";
import { PAGE_KEYS, PAGE_PICTURE, type PageKey } from "@/lib/pages";

import { CtaLink } from "./cta-link";
import { PageShell } from "./page-shell";
import { Reveal } from "./reveal";
import { PANEL, TEXTURE, TEXTURE_FILL } from "./textures";
import { WindowsIcon } from "./windows-icon";

type ItemCopy = {
	lead: string;
	open: string;
	collapsed: string | null;
	settings: string | null;
};

/** What the collapsed pill shows for the pages that have an activity; a page without one has no entry. */
const COLLAPSED_EXAMPLE: Partial<Record<PageKey, ReactNode>> = {
	media: <WavePill />,
	shelf: <ShelfPill />,
	calendar: <EventPill />,
	timer: (
		<>
			<TimerPill />
			<TimerDonePill />
		</>
	),
	prayer: <PrayerPill />,
	weather: <RainPill />,
	clipboard: <CopiedPill />,
	devices: <DeviceBatteryPill />,
};

/** One row of the page sheet: a label on the left axis, the text and an optional picture on the picture's axis. */
function StateRow({
	label,
	text,
	muted = false,
	children,
}: {
	label: string;
	text: string;
	muted?: boolean;
	children?: ReactNode;
}) {
	return (
		<section className="grid grid-cols-[repeat(3,minmax(0,1fr))] gap-x-12 gap-y-3 border-b border-border py-9 max-[900px]:grid-cols-[minmax(0,1fr)] max-[640px]:py-7">
			<h2 className="m-0 text-[15px] font-medium text-muted-foreground">{label}</h2>
			<div className="col-span-2 flex flex-col gap-6 max-[900px]:col-span-1">
				<p
					className={`m-0 max-w-[58ch] text-pretty ${muted ? "text-[17px] leading-[1.6] text-muted-foreground" : "text-[clamp(18px,1.6vw,22px)] leading-[1.5] tracking-[-0.01em]"}`}
				>
					{text}
				</p>
				{children}
			</div>
		</section>
	);
}

/** One page of the notch: its picture, what it shows open and collapsed, and what Settings holds for it. Every page uses this one layout. */
export function PageDetail({ pageKey }: { pageKey: PageKey }) {
	const t = useTranslations();
	const item = t.raw(`pageDetail.items.${pageKey}`) as ItemCopy;
	const index = PAGE_KEYS.indexOf(pageKey);
	const previous = index > 0 ? PAGE_KEYS[index - 1] : null;
	const next = index < PAGE_KEYS.length - 1 ? PAGE_KEYS[index + 1] : null;
	const title = t(`grid.${pageKey}`);
	const example = COLLAPSED_EXAMPLE[pageKey];

	return (
		<PageShell>
			<Reveal className="grid grid-cols-[repeat(3,minmax(0,1fr))] items-center gap-x-12 gap-y-8 max-[900px]:grid-cols-[minmax(0,1fr)]">
				<div>
					<p className="m-0 flex flex-wrap items-center gap-x-3 gap-y-1 text-[14px] text-muted-foreground">
						<span className="font-mono">
							{t("pageDetail.position", {
								number: String(index + 1).padStart(2, "0"),
								total: PAGE_KEYS.length,
							})}
						</span>
					</p>
					<h1 className="mt-3 text-[clamp(30px,3vw,40px)] leading-[1.18] font-medium tracking-[-0.03em] text-balance">
						{title}
					</h1>
					<p className="mt-4 max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground">
						{item.lead}
					</p>
				</div>
				<div className="col-span-2 max-[900px]:col-span-1">
					{/* eslint-disable-next-line @next/next/no-img-element */}
					<img
						src={`/pages/${pageKey}.jpg`}
						width={PAGE_PICTURE[pageKey].width}
						height={PAGE_PICTURE[pageKey].height}
						alt={t("pageDetail.pictureAlt", { page: title })}
						decoding="async"
						className="block h-auto w-full rounded-[14px]"
					/>
				</div>
			</Reveal>

			<Reveal className="mt-16 border-t border-border">
				<StateRow label={t("pageDetail.openTitle")} text={item.open} />
				<StateRow
					label={t("pageDetail.collapsedTitle")}
					text={item.collapsed ?? t("pageDetail.noCollapsed")}
					muted={!item.collapsed}
				>
					{example && (
						<div>
							<div
								className={`${PANEL} ${TEXTURE.universtock} ${TEXTURE_FILL} min-h-[132px] items-start`}
							>
								<div className="flex flex-col items-center gap-3">{example}</div>
							</div>
							<p className="mt-3 mb-0 text-[14px] text-muted-foreground">
								{t("pageDetail.exampleNote")}
							</p>
						</div>
					)}
				</StateRow>
				<StateRow
					label={t("pageDetail.settingsTitle")}
					text={item.settings ?? t("pageDetail.noSettings")}
					muted={!item.settings}
				/>
			</Reveal>

			<nav
				aria-label={t("pageDetail.navLabel")}
				className="mt-10 grid grid-cols-[repeat(2,minmax(0,1fr))] gap-6 max-[640px]:grid-cols-[minmax(0,1fr)]"
			>
				{previous ? (
					<Link href={`/pages/${previous}`} className="flex flex-col gap-1 no-underline">
						<span className="text-[14px] text-muted-foreground">← {t("pageDetail.previous")}</span>
						<span className="text-[17px] font-medium">{t(`grid.${previous}`)}</span>
					</Link>
				) : (
					<span />
				)}
				{next && (
					<Link
						href={`/pages/${next}`}
						className="flex flex-col gap-1 text-right no-underline max-[640px]:text-left"
					>
						<span className="text-[14px] text-muted-foreground">{t("pageDetail.next")} →</span>
						<span className="text-[17px] font-medium">{t(`grid.${next}`)}</span>
					</Link>
				)}
			</nav>

			<div className="mt-16 flex flex-wrap items-center justify-between gap-6 rounded-[14px] bg-card p-6 [box-shadow:0_0_0_1px_var(--border)]">
				<div>
					<h2 className="m-0 text-[20px] font-medium">{t("pageDetail.ctaTitle")}</h2>
					<p className="mt-1 text-[15px] text-muted-foreground">{t("pageDetail.ctaText")}</p>
				</div>
				<CtaLink href="/download" size="lg" variant="primary">
					<WindowsIcon />
					{t("hero.cta")}
				</CtaLink>
			</div>
		</PageShell>
	);
}
