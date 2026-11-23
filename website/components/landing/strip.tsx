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
		<div key="weather" style={{ zoom: 0.92 }}>
			<WeatherCells />
		</div>,
	],
	["todo", <TodoList key="todo" />],
	["shelf", <ShelfGrid key="shelf" />],
	[
		"stats",
		<div key="stats" style={{ zoom: 0.78 }}>
			<StatsGrid />
		</div>,
	],
];

/** The headline and four-cell strip right below the proof row. */
export function Strip() {
	const t = useTranslations();

	return (
		<section id="features" style={{ paddingTop: "clamp(64px, 7vw, 96px)" }}>
			<div className="wrap" style={{ marginBottom: 56 }}>
				<TwoTone first={t("all.a")} second={t("all.b")} />
			</div>
			<div className="wrap">
				<div className="strip">
					{CELLS.map(([key, visual]) => (
						<div key={key} className="cell">
							<h3 style={{ margin: 0, fontSize: 21, fontWeight: 500, letterSpacing: "-0.015em" }}>
								{t(`strip.${key}.t`)}
							</h3>
							<p
								style={{
									margin: "10px 0 0",
									fontSize: 16,
									lineHeight: 1.55,
									color: "var(--muted-foreground)",
								}}
							>
								{t(`strip.${key}.d`)}
							</p>
							<div
								className="sv"
								style={{
									flex: 1,
									minHeight: 240,
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
									paddingTop: 28,
								}}
							>
								{visual}
							</div>
						</div>
					))}
				</div>
			</div>
		</section>
	);
}
