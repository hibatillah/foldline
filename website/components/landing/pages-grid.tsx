import {
	BatteryMedium,
	CalendarDays,
	ChartColumn,
	ClipboardList,
	CloudDrizzle,
	Inbox,
	ListChecks,
	MoonStar,
	Music,
	Timer,
	type LucideIcon,
} from "lucide-react";
import { useTranslations } from "next-intl";

import { PageTile } from "./page-tile";
import { TwoTone } from "./two-tone";

type PageKey =
	| "media"
	| "shelf"
	| "calendar"
	| "timer"
	| "todo"
	| "prayer"
	| "weather"
	| "stats"
	| "clipboard"
	| "devices";

const PAGES: Array<[PageKey, LucideIcon]> = [
	["media", Music],
	["shelf", Inbox],
	["calendar", CalendarDays],
	["timer", Timer],
	["todo", ListChecks],
	["prayer", MoonStar],
	["weather", CloudDrizzle],
	["stats", ChartColumn],
	["clipboard", ClipboardList],
	["devices", BatteryMedium],
];

/** The pictures of `public/pages`, taken from the final designs on a dither background; each is as wide as its notch plus a margin. */
const PAGE_PICTURE: Record<PageKey, { width: number; height: number }> = {
	media: { width: 734, height: 184 },
	shelf: { width: 660, height: 184 },
	calendar: { width: 660, height: 250 },
	timer: { width: 442, height: 184 },
	todo: { width: 660, height: 242 },
	prayer: { width: 660, height: 184 },
	weather: { width: 660, height: 184 },
	stats: { width: 844, height: 184 },
	clipboard: { width: 660, height: 277 },
	devices: { width: 592, height: 184 },
};

/** A hover card shows a picture at this share of its size, so every notch has the same scale. */
const CARD_SCALE = 0.8;

const COLUMNS = 5;

/** Where the hover card sits above its tile: the first column keeps its left edge, the last its right edge. */
function cardAlignment(index: number) {
	const column = index % COLUMNS;
	if (column === 0) return "left-0";
	if (column === COLUMNS - 1) return "right-0";
	return "left-1/2 -translate-x-1/2";
}

/** The ten pages as a grid of square tiles; on a pointer device each tile shows its page when hovered. */
export function PagesGrid() {
	const t = useTranslations();

	return (
		<section className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] pt-[clamp(120px,13vw,200px)] max-[640px]:pt-24">
			<div className="grid grid-cols-[repeat(2,minmax(0,1fr))] gap-x-12 gap-y-6 mb-[72px] max-[900px]:grid-cols-[minmax(0,1fr)] max-[640px]:mb-10 max-[640px]:gap-4">
				<div>
					<TwoTone first={t("gridHead.a")} second={t("gridHead.b")} />
				</div>
				<div>
					<p className="max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground m-0 max-[640px]:mt-5">
						{t("gridHead.p")}
					</p>
				</div>
			</div>
			<div className="grid grid-cols-[repeat(5,minmax(0,1fr))] gap-[18px] max-[1000px]:grid-cols-[repeat(3,minmax(0,1fr))] max-[640px]:grid-cols-[repeat(2,minmax(0,1fr))] max-[640px]:gap-3">
				{PAGES.map(([key, Icon], index) => (
					<PageTile
						key={key}
						picture={`/pages/${key}.jpg`}
						width={PAGE_PICTURE[key].width * CARD_SCALE + 12}
						height={PAGE_PICTURE[key].height}
						alignment={cardAlignment(index)}
						className="group relative box-border flex aspect-square flex-col justify-between rounded-[14px] bg-card p-5 [box-shadow:0_0_0_1px_var(--border)] transition-shadow duration-200 hover:[box-shadow:0_0_0_1px_color-mix(in_oklch,var(--primary)_45%,transparent)] max-[640px]:p-4"
					>
						<div className="flex justify-between items-start">
							<span className="w-[44px] h-[44px] rounded-[11px] bg-[color:color-mix(in_oklch,_var(--primary)_10%,_transparent)] text-[color:oklch(0.42_0.19_263)] flex items-center justify-center">
								<Icon aria-hidden="true" size={22} strokeWidth={2} />
							</span>
							<span className="font-mono text-[13px] text-muted-foreground">
								{String(index + 1).padStart(2, "0")}
							</span>
						</div>
						<div className="flex flex-col gap-[4px]">
							<span className="text-[16px] font-medium">{t(`grid.${key}`)}</span>
							<span className="text-[13px] leading-[1.45] text-muted-foreground">
								{t(`gridSub.${key}`)}
							</span>
						</div>
					</PageTile>
				))}
			</div>
		</section>
	);
}
