"use client";

import { motion, useInView, useReducedMotion } from "motion/react";
import { useEffect, useRef, useState } from "react";

type PartKey = "edge" | "pills" | "clock" | "status" | "page" | "host";
type Label = { t: string; d: string };

/** Scale from the notch's own pixels (see docs/reference/layout.md) to the drawing. */
const SCALE = 1.5;
const CENTER_X = 500;
/** The drawing's box is this wide; labels sit outside it, in the margins of the section. */
const VIEW_WIDTH = 620;
const VIEW_HEIGHT = 350;
const VIEW_LEFT = CENTER_X - VIEW_WIDTH / 2;
const VIEW_RIGHT = CENTER_X + VIEW_WIDTH / 2;

const PAGE_WIDTH = 300;
const SIDE_PADDING = 16;
const TOP_PADDING = 10;
const HEADER_HEIGHT = 16;
const HEADER_GAP = 10;
const PAGE_HEIGHT = 86;
const BOTTOM_PADDING = 14;
const FLARE = 12;
const BOTTOM_RADIUS = 22;
const PAGE_DOT_COUNT = 10;
const DOT_SIZE = 6;
const DOT_GAP = 4;
const ACTIVE_DOT_WIDTH = 16;
/** How many pages the open dot walks through while the drawing is on screen. */
const DEMO_PAGES = 4;

const notchWidth = (PAGE_WIDTH + SIDE_PADDING * 2) * SCALE;
const notchHeight =
	(TOP_PADDING + HEADER_HEIGHT + HEADER_GAP + PAGE_HEIGHT + BOTTOM_PADDING) * SCALE;
const left = CENTER_X - notchWidth / 2;
const right = left + notchWidth;
const top = 96;
const bottom = top + notchHeight;
const flare = FLARE * SCALE;
const radius = BOTTOM_RADIUS * SCALE;

const innerLeft = left + SIDE_PADDING * SCALE;
const innerRight = right - SIDE_PADDING * SCALE;
const headerTop = top + TOP_PADDING * SCALE;
const headerHeight = HEADER_HEIGHT * SCALE;
const headerCenter = headerTop + headerHeight / 2;
const pageTop = headerTop + headerHeight + HEADER_GAP * SCALE;
const pageHeight = PAGE_HEIGHT * SCALE;
const pageWidth = innerRight - innerLeft;
const dotHeight = DOT_SIZE * SCALE;

/** The notch outline: a flare at each top corner and rounded bottom corners. */
const NOTCH_PATH = [
	`M ${left - flare} ${top}`,
	`A ${flare} ${flare} 0 0 1 ${left} ${top + flare}`,
	`L ${left} ${bottom - radius}`,
	`A ${radius} ${radius} 0 0 0 ${left + radius} ${bottom}`,
	`L ${right - radius} ${bottom}`,
	`A ${radius} ${radius} 0 0 0 ${right} ${bottom - radius}`,
	`L ${right} ${top + flare}`,
	`A ${flare} ${flare} 0 0 1 ${right + flare} ${top}`,
	"Z",
].join(" ");

const EASE_OUT = [0.16, 1, 0.3, 1] as const;

/** The leader runs from `from` on the notch to the label's edge; `labelY` is the label's top. A "top" label sits above the screen edge and its leader runs up. */
const CALLOUTS: Record<
	PartKey,
	{ from: [number, number]; to: number; side: "left" | "right" | "top"; labelY: number }
> = {
	edge: { from: [VIEW_LEFT, top], to: VIEW_LEFT, side: "left", labelY: top - 50 },
	pills: {
		from: [innerLeft - 6, headerCenter],
		to: VIEW_LEFT - 4,
		side: "left",
		labelY: headerCenter - 18,
	},
	clock: { from: [CENTER_X, headerTop - 3], to: top - 20, side: "top", labelY: top - 70 },
	status: {
		from: [innerRight + 4, headerCenter],
		to: VIEW_RIGHT + 4,
		side: "right",
		labelY: headerCenter - 18,
	},
	page: {
		from: [innerRight - 14, pageTop + pageHeight / 2],
		to: VIEW_RIGHT + 4,
		side: "right",
		labelY: pageTop + pageHeight / 2 - 18,
	},
	host: {
		from: [right - 8, bottom - 14],
		to: VIEW_RIGHT + 4,
		side: "right",
		labelY: bottom - 28,
	},
};

/** How a label sits against its leader, and where it slides in from. */
const LABEL_PLACEMENT = {
	left: { left: 0, translate: "calc(-100% - 10px)", textAlign: "right", x: -10, y: 0 },
	right: { left: "100%", translate: "10px", textAlign: "left", x: 10, y: 0 },
	top: { left: "50%", translate: "-50% 0", textAlign: "center", x: 0, y: 8 },
} as const;

const PARTS: PartKey[] = ["edge", "clock", "pills", "status", "page", "host"];

/** The pin and the gear of the notch's own icons (24 px drawings), set small in the header. */
const PIN_PATH =
	"M12 17v5M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z";
const GEAR_PATH = "M12 2.5 20.2 7.25v9.5L12 21.5 3.8 16.75v-9.5z";
const ICON_SCALE = 0.7;
const ICON_SIZE = 24 * ICON_SCALE;
const dotsWidth =
	((PAGE_DOT_COUNT - 1) * DOT_SIZE + ACTIVE_DOT_WIDTH + (PAGE_DOT_COUNT - 1) * DOT_GAP) * SCALE;
const statusWidth = 124;

/** The notch drawn from the front, its parts outlined and named; the labels draw in when it scrolls into view. */
export function AnatomyDiagram({
	labels,
	legend,
}: {
	labels: Record<PartKey, Label>;
	legend: string;
}) {
	const containerRef = useRef<HTMLDivElement>(null);
	const reducedMotion = useReducedMotion();
	const everInView = useInView(containerRef, { once: true, margin: "0px 0px -120px 0px" });
	const inView = useInView(containerRef);
	const [activePart, setActivePart] = useState<PartKey | null>(null);
	const [activePage, setActivePage] = useState(0);
	const shown = everInView || Boolean(reducedMotion);

	// The open page dot moves on while the drawing is on screen, so the header reads as alive.
	useEffect(() => {
		if (!inView || reducedMotion) return;
		const timer = setInterval(() => setActivePage((page) => (page + 1) % DEMO_PAGES), 1800);
		return () => clearInterval(timer);
	}, [inView, reducedMotion]);

	const outlineOpacity = (part: PartKey) => {
		if (activePart === null) return 0.75;
		return activePart === part ? 1 : 0.25;
	};
	const outlineFill = (part: PartKey) =>
		activePart === part ? "color-mix(in oklch, var(--primary) 14%, transparent)" : "transparent";
	const draw = (delay: number) => ({
		initial: reducedMotion ? false : { pathLength: 0 },
		animate: shown ? { pathLength: 1 } : undefined,
		transition: { duration: 0.4, delay, ease: EASE_OUT },
	});

	return (
		<div className="min-[900px]:px-[19%]">
			<div
				ref={containerRef}
				className="relative -mt-10 -mb-5 w-full min-[900px]:mt-0 min-[900px]:mb-0"
				style={{ aspectRatio: `${VIEW_WIDTH} / ${VIEW_HEIGHT}` }}
			>
				<svg
					viewBox={`${VIEW_LEFT} 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
					className="absolute inset-0 size-full overflow-visible"
					role="img"
					aria-label={legend}
				>
					{/* The top edge of the screen, running on past the drawing */}
					<motion.line
						x1={0}
						x2={CENTER_X * 2}
						y1={top}
						y2={top}
						stroke="currentColor"
						className="text-foreground/45 max-[900px]:hidden"
						strokeWidth={1.5}
						{...draw(0)}
					/>

					<motion.g
						initial={reducedMotion ? false : { opacity: 0, y: -18 }}
						animate={shown ? { opacity: 1, y: 0 } : undefined}
						transition={{ duration: 0.7, delay: 0.15, ease: EASE_OUT }}
					>
						<path d={NOTCH_PATH} fill="#000" stroke="rgba(128,128,128,0.4)" strokeWidth={1} />

						{/* Header: page dots, clock, status icons, pin and gear */}
						{Array.from({ length: PAGE_DOT_COUNT }, (_, index) => {
							const active = index === activePage;
							const offset =
								index * (DOT_SIZE + DOT_GAP) +
								(index > activePage ? ACTIVE_DOT_WIDTH - DOT_SIZE : 0);

							return (
								<motion.rect
									key={index}
									y={headerCenter - dotHeight / 2}
									height={dotHeight}
									rx={dotHeight / 2}
									fill={active ? "#e6e6eb" : "#3a3a44"}
									initial={false}
									animate={{
										x: innerLeft + offset * SCALE,
										width: (active ? ACTIVE_DOT_WIDTH : DOT_SIZE) * SCALE,
									}}
									transition={{ duration: 0.45, ease: EASE_OUT }}
								/>
							);
						})}
						<text
							x={CENTER_X}
							y={headerCenter}
							textAnchor="middle"
							dominantBaseline="central"
							fill="#fff"
							fontSize={11 * SCALE}
							fontWeight={500}
							className="font-sans"
						>
							15:05
						</text>
						{(
							[
								["#2f9e44", 106],
								["#f08c00", 84],
								["#4c9aff", 62],
							] as const
						).map(([color, offset]) => (
							<circle
								key={color}
								cx={innerRight - offset}
								cy={headerCenter}
								r={6.5}
								fill="none"
								stroke={color}
								strokeWidth={2.2}
							/>
						))}
						<g
							transform={`translate(${innerRight - 38 - ICON_SIZE / 2} ${headerCenter - ICON_SIZE / 2}) scale(${ICON_SCALE})`}
							fill="none"
							stroke="#a1a1aa"
							strokeWidth={2}
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d={PIN_PATH} />
						</g>
						<g
							transform={`translate(${innerRight - ICON_SIZE} ${headerCenter - ICON_SIZE / 2}) scale(${ICON_SCALE})`}
							fill="none"
							stroke="#a1a1aa"
							strokeWidth={2}
							strokeLinejoin="round"
						>
							<path d={GEAR_PATH} />
							<circle cx={12} cy={12} r={3} />
						</g>

						{/* Page: a media page's content as stand-ins */}
						<rect
							x={innerLeft + 12}
							y={pageTop + 12}
							width={pageHeight - 24}
							height={pageHeight - 24}
							rx={10}
							fill="#fff"
							fillOpacity={0.12}
						/>
						<rect
							x={innerLeft + pageHeight}
							y={pageTop + 22}
							width={150}
							height={10}
							rx={5}
							fill="#fff"
							fillOpacity={0.85}
						/>
						<rect
							x={innerLeft + pageHeight}
							y={pageTop + 40}
							width={96}
							height={8}
							rx={4}
							fill="#fff"
							fillOpacity={0.35}
						/>
						<rect
							x={innerLeft + pageHeight}
							y={pageTop + 70}
							width={pageWidth - pageHeight - 24}
							height={6}
							rx={3}
							fill="#fff"
							fillOpacity={0.18}
						/>
						<rect
							x={innerLeft + pageHeight}
							y={pageTop + 70}
							width={(pageWidth - pageHeight - 24) * 0.45}
							height={6}
							rx={3}
							fill="#fff"
							fillOpacity={0.85}
						/>
						{[0, 1, 2].map((control) => (
							<circle
								key={control}
								cx={innerLeft + pageHeight + 12 + control * 30}
								cy={pageTop + 100}
								r={8}
								fill="#fff"
								fillOpacity={0.5}
							/>
						))}
					</motion.g>

					{/* Outlines of the host, the pills, the clock, the status bar and the page */}
					<motion.g
						initial={reducedMotion ? false : { opacity: 0 }}
						animate={shown ? { opacity: 1 } : undefined}
						transition={{ duration: 0.3, delay: 0.3 }}
					>
						<motion.path
							d={NOTCH_PATH}
							fill="none"
							stroke="var(--primary)"
							strokeWidth={1.5}
							strokeDasharray="5 5"
							animate={{ opacity: outlineOpacity("host") }}
							style={{ fill: outlineFill("host") }}
						/>
						<motion.rect
							x={innerLeft - 5}
							y={headerTop - 3}
							width={dotsWidth + 10}
							height={headerHeight + 6}
							rx={8}
							fill="none"
							stroke="var(--primary)"
							strokeWidth={1.5}
							strokeDasharray="5 5"
							style={{ fill: outlineFill("pills") }}
							animate={{ opacity: outlineOpacity("pills") }}
						/>
						<motion.rect
							x={CENTER_X - 36}
							y={headerTop - 3}
							width={72}
							height={headerHeight + 6}
							rx={8}
							fill="none"
							stroke="var(--primary)"
							strokeWidth={1.5}
							strokeDasharray="5 5"
							style={{ fill: outlineFill("clock") }}
							animate={{ opacity: outlineOpacity("clock") }}
						/>
						<motion.rect
							x={innerRight - statusWidth}
							y={headerTop - 3}
							width={statusWidth + 4}
							height={headerHeight + 6}
							rx={8}
							fill="none"
							stroke="var(--primary)"
							strokeWidth={1.5}
							strokeDasharray="5 5"
							style={{ fill: outlineFill("status") }}
							animate={{ opacity: outlineOpacity("status") }}
						/>
						<motion.rect
							x={innerLeft - 4}
							y={pageTop - 4}
							width={pageWidth + 8}
							height={pageHeight + 8}
							rx={12}
							fill="none"
							stroke="var(--primary)"
							strokeWidth={1.5}
							strokeDasharray="5 5"
							style={{ fill: outlineFill("page") }}
							animate={{ opacity: outlineOpacity("page") }}
						/>
					</motion.g>

					{/* Leaders from the parts to their labels */}
					{PARTS.filter((part) => part !== "edge").map((part, index) => {
						const callout = CALLOUTS[part];
						const [fromX, fromY] = callout.from;

						return (
							<g key={part} className="max-[900px]:hidden">
								<motion.line
									x1={fromX}
									y1={fromY}
									x2={callout.side === "top" ? fromX : callout.to}
									y2={callout.side === "top" ? callout.to : fromY}
									stroke="currentColor"
									className="text-foreground/50"
									strokeWidth={1}
									{...draw(0.45 + index * 0.05)}
								/>
								<motion.circle
									cx={fromX}
									cy={fromY}
									r={3}
									fill="var(--primary)"
									initial={reducedMotion ? false : { scale: 0 }}
									animate={shown ? { scale: 1 } : undefined}
									style={{ transformOrigin: `${fromX}px ${fromY}px` }}
									transition={{
										delay: 0.45 + index * 0.05,
										type: "spring",
										stiffness: 400,
										damping: 18,
									}}
								/>
							</g>
						);
					})}
				</svg>

				{PARTS.map((part, index) => {
					const callout = CALLOUTS[part];
					const label = labels[part];
					const placement = LABEL_PLACEMENT[callout.side];

					return (
						<motion.div
							key={part}
							aria-hidden="true"
							className="absolute w-[30%] cursor-default max-[900px]:hidden"
							style={{
								top: `${(callout.labelY / VIEW_HEIGHT) * 100}%`,
								left: placement.left,
								translate: placement.translate,
								textAlign: placement.textAlign,
							}}
							initial={reducedMotion ? false : { opacity: 0, x: placement.x, y: placement.y }}
							animate={shown ? { opacity: 1, x: 0, y: 0 } : undefined}
							transition={{ duration: 0.3, delay: 0.5 + index * 0.05, ease: EASE_OUT }}
							onMouseEnter={() => setActivePart(part)}
							onMouseLeave={() => setActivePart(null)}
						>
							<p className="m-0 text-[14px] leading-[1.3] font-medium">{label.t}</p>
							<p className="m-0 mt-0.5 text-[13px] leading-[1.4] text-muted-foreground">
								{label.d}
							</p>
						</motion.div>
					);
				})}
			</div>

			<dl className="m-0 mt-4 grid grid-cols-2 gap-x-6 gap-y-5 min-[900px]:sr-only max-[640px]:grid-cols-1">
				{PARTS.map((part) => (
					<div key={part} className="border-t border-border pt-3">
						<dt className="text-[15px] font-medium">{labels[part].t}</dt>
						<dd className="m-0 mt-1 text-[14px] leading-[1.5] text-muted-foreground">
							{labels[part].d}
						</dd>
					</div>
				))}
			</dl>
		</div>
	);
}
