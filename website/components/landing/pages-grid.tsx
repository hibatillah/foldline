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

/** The ten pages as a grid of square tiles. */
export function PagesGrid() {
	const t = useTranslations();

	return (
		<section className="wrap feature">
			<div className="fhead">
				<div>
					<TwoTone first={t("gridHead.a")} second={t("gridHead.b")} />
				</div>
				<div>
					<p className="body" style={{ margin: 0 }}>
						{t("gridHead.p")}
					</p>
				</div>
			</div>
			<div className="brackets">
				{PAGES.map(([key, Icon], index) => (
					<div key={key} className="bracket">
						<div
							style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}
						>
							<span
								style={{
									width: 44,
									height: 44,
									borderRadius: 11,
									background: "color-mix(in oklch, var(--primary) 10%, transparent)",
									color: "oklch(0.42 0.19 263)",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								<Icon aria-hidden="true" size={22} strokeWidth={2} />
							</span>
							<span
								style={{
									fontFamily: "var(--font-mono)",
									fontSize: 13,
									color: "var(--muted-foreground)",
								}}
							>
								{String(index + 1).padStart(2, "0")}
							</span>
						</div>
						<div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
							<span style={{ fontSize: 16, fontWeight: 500 }}>{t(`grid.${key}`)}</span>
							<span style={{ fontSize: 13, lineHeight: 1.45, color: "var(--muted-foreground)" }}>
								{t(`gridSub.${key}`)}
							</span>
						</div>
					</div>
				))}
			</div>
		</section>
	);
}
