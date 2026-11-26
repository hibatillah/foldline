import { useTranslations } from "next-intl";
import type { ReactElement, ReactNode } from "react";

import {
	CopiedPill,
	LyricsPill,
	DeviceBatteryPill,
	EventPill,
	PrayerPill,
	RainPill,
	ShelfPill,
	TimerPill,
	WavePill,
} from "@/components/notch/pill";
import { NOTCH_PICTURE, PAGE_KEYS, type PageKey } from "@/lib/pages";

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

/** What the collapsed pill shows for the pages that have an activity, one card each; a page without one has no entry. */
const COLLAPSED_EXAMPLES: Partial<Record<PageKey, ReactElement[]>> = {
	media: [<WavePill key="wave" />, <LyricsPill key="lyrics" />],
	shelf: [<ShelfPill key="shelf" />],
	calendar: [<EventPill key="event" />],
	timer: [<TimerPill key="timer" />],
	prayer: [<PrayerPill key="prayer" />],
	weather: [<RainPill key="rain" />],
	clipboard: [<CopiedPill key="copied" />],
	devices: [<DeviceBatteryPill key="battery" />],
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

/** The open notch of a page at its own size, centered on a card of the same width as the collapsed ones. */
function NotchCard({ pageKey, alt }: { pageKey: PageKey; alt: string }) {
	const { width, height } = NOTCH_PICTURE[pageKey];

	return (
		<div
			className={`${PANEL} ${TEXTURE.universtock} ${TEXTURE_FILL} items-start`}
			style={{ minHeight: height }}
		>
			{/* eslint-disable-next-line @next/next/no-img-element */}
			<img
				src={`/pages/notch/${pageKey}.webp`}
				width={width}
				height={height}
				alt={alt}
				decoding="async"
				className="block h-auto max-w-full"
			/>
		</div>
	);
}

/** One page of the notch: its title and lead, then what it shows open and collapsed, and what Settings holds for it. Every page uses this one layout. */
export function PageDetail({ pageKey }: { pageKey: PageKey }) {
	const t = useTranslations();
	const item = t.raw(`pageDetail.items.${pageKey}`) as ItemCopy;
	const index = PAGE_KEYS.indexOf(pageKey);
	const title = t(`grid.${pageKey}`);
	const examples = COLLAPSED_EXAMPLES[pageKey];

	return (
		<PageShell>
			<Reveal>
				<p className="m-0 font-mono text-[14px] text-muted-foreground">
					{t("pageDetail.position", {
						number: String(index + 1).padStart(2, "0"),
						total: PAGE_KEYS.length,
					})}
				</p>
				<h1 className="mt-3 text-[clamp(30px,3vw,40px)] leading-[1.18] font-medium tracking-[-0.03em] text-balance">
					{title}
				</h1>
				<p className="mt-4 text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-pretty text-muted-foreground">
					{item.lead}
				</p>
			</Reveal>

			<Reveal className="mt-16 border-t border-border">
				<StateRow label={t("pageDetail.openTitle")} text={item.open}>
					<NotchCard pageKey={pageKey} alt={t("pageDetail.pictureAlt", { page: title })} />
				</StateRow>
				<StateRow
					label={t("pageDetail.collapsedTitle")}
					text={item.collapsed ?? t("pageDetail.noCollapsed")}
					muted={!item.collapsed}
				>
					{examples && (
						<div className="flex flex-col gap-3">
							{examples.map((pill) => (
								<div
									key={pill.key}
									className={`${PANEL} ${TEXTURE.universtock} ${TEXTURE_FILL} min-h-[104px] items-start`}
								>
									{pill}
								</div>
							))}
							<p className="m-0 text-[14px] text-muted-foreground">{t("pageDetail.exampleNote")}</p>
						</div>
					)}
				</StateRow>
				<StateRow
					label={t("pageDetail.settingsTitle")}
					text={item.settings ?? t("pageDetail.noSettings")}
					muted={!item.settings}
				/>
			</Reveal>

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
