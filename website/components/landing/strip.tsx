import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { ShelfGrid } from "./elements/shelf-grid";
import { StatsGrid } from "./elements/stats-grid";
import { TodoList } from "./elements/todo-list";
import { WeatherCells } from "./elements/weather-cells";
import { TwoTone } from "./two-tone";

type StripKey = "weather" | "todo" | "shelf" | "stats";

const CELLS: Array<[StripKey, ReactNode]> = [
	[
		"weather",
		<div key="weather" className="[zoom:0.92]">
			<WeatherCells />
		</div>,
	],
	["todo", <TodoList key="todo" />],
	["shelf", <ShelfGrid key="shelf" />],
	[
		"stats",
		<div key="stats" className="[zoom:0.78]">
			<StatsGrid />
		</div>,
	],
];

/** The headline and four-cell strip right below the proof row. */
export function Strip() {
	const t = useTranslations();

	return (
		<section id="features" className="pt-[clamp(64px,_7vw,_96px)]">
			<div className="mx-auto mb-14 box-border max-w-[1280px] px-[clamp(16px,5vw,80px)]">
				<TwoTone first={t("all.a")} second={t("all.b")} />
			</div>
			<div className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)]">
				<div className="grid grid-cols-[repeat(4,minmax(0,1fr))] border-y border-border max-[1000px]:grid-cols-[repeat(2,minmax(0,1fr))] max-[640px]:grid-cols-[minmax(0,1fr)] max-[640px]:border-b-0">
					{CELLS.map(([key, visual]) => (
						<div
							key={key}
							className="flex flex-col border-l border-border bg-[color-mix(in_oklch,var(--muted)_55%,transparent)] p-8 last:border-r max-[1000px]:border-b max-[640px]:border-l-0 max-[640px]:border-b max-[640px]:px-0 max-[640px]:py-7 max-[640px]:last:border-r-0"
						>
							<h3 className="m-0 text-[21px] font-medium tracking-[-0.015em]">
								{t(`strip.${key}.t`)}
							</h3>
							<p className="m-[10px_0_0] text-[16px] leading-[1.55] text-muted-foreground">
								{t(`strip.${key}.d`)}
							</p>
							<div className="flex min-h-[240px] flex-1 items-center justify-center pt-7 max-[640px]:min-h-0 max-[640px]:pt-6">
								{visual}
							</div>
						</div>
					))}
				</div>
			</div>
		</section>
	);
}
