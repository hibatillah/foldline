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

import { Link } from "@/i18n/navigation";
import type { PageKey } from "@/lib/pages";

import { TwoTone } from "./two-tone";

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

/** The ten pages as a grid of square tiles, each linking to its page. */
export function PagesGrid() {
	const t = useTranslations();

	return (
		<section
			id="pages"
			className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] pt-[clamp(120px,13vw,200px)] max-[640px]:pt-24"
		>
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
					<div
						key={key}
						className="relative box-border flex aspect-square flex-col justify-between rounded-[14px] bg-card p-5 [box-shadow:0_0_0_1px_var(--border)] transition-shadow duration-200 hover:[box-shadow:0_0_0_1px_color-mix(in_oklch,var(--primary)_45%,transparent)] max-[640px]:p-4"
					>
						<Link
							href={`/pages/${key}`}
							aria-label={t(`grid.${key}`)}
							className="absolute inset-0 z-10 rounded-[14px]"
						/>
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
					</div>
				))}
			</div>
		</section>
	);
}
