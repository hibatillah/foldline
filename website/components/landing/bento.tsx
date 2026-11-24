import {
	ArrowBigUp,
	Bluetooth,
	CloudRain,
	Download,
	Headphones,
	Keyboard,
	Mouse,
	Search,
	Sun,
	Volume2,
	VolumeX,
} from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { BatteryRingIcon } from "@/components/notch/icons";
import { MediaNotch } from "@/components/notch/media";
import { Flares, NotchHeader } from "@/components/notch/open-notch";
import { ShelfNotch } from "@/components/notch/shelf";
import type en from "@/messages/en.json";

import { Reveal, RevealGroup, RevealItem } from "./reveal";
import { TEXTURE } from "./textures";
import { TwoTone } from "./two-tone";
import { windowsLogoPath } from "./windows-icon";

type Messages = typeof en;

/** What every tile shares; on a desktop the grid gives each its height, below that a tile hugs its content. */
const TILE_BASE =
	"relative flex h-full flex-col overflow-hidden rounded-[14px] p-4 [box-shadow:inset_0_0_0_1px_color-mix(in_oklch,var(--foreground)_9%,transparent)]";
const TILE_MIN_HEIGHT = "min-[640px]:min-h-[200px] min-[1000px]:min-h-0";
const LIGHT_TILE = `${TILE_BASE} ${TILE_MIN_HEIGHT} bg-[oklch(0.965_0.006_250)] text-[#18181b]`;
const BLUE_TILE = `${TILE_BASE} ${TILE_MIN_HEIGHT} bg-[oklch(0.95_0.025_262)] text-[#18181b]`;
const WEATHER_TILE = `${TILE_BASE} ${TILE_MIN_HEIGHT} bg-[linear-gradient(160deg,#4c9aff_0%,#1d4ed8_100%)] text-white`;
const PICTURE_TILE = `${TILE_BASE} ${TILE_MIN_HEIGHT} bg-cover bg-no-repeat bg-[color:oklch(0.2_0.05_255)] text-white`;

/** How much each real notch drawing shrinks: a phone, a tablet, a narrow desktop and a wide desktop. */
const ZOOM_MEDIA =
	"[zoom:0.557] min-[640px]:[zoom:0.9] min-[1000px]:[zoom:0.56] min-[1280px]:[zoom:0.7]";
const ZOOM_SHELF =
	"[zoom:0.6] min-[640px]:[zoom:0.9] min-[1000px]:[zoom:0.47] min-[1280px]:[zoom:0.6]";
/** Smaller than the tile by more than its corner radius, so the flares never meet the rounded corners. */
const ZOOM_MINI =
	"[zoom:0.85] min-[640px]:[zoom:0.75] min-[1000px]:[zoom:0.34] min-[1280px]:[zoom:0.45]";

const EYEBROW = "font-mono text-[10px] uppercase tracking-[0.08em]";
const TITLE = "m-0 text-[15px] font-medium leading-[1.2] tracking-[-0.01em]";
const DESC = "m-0 text-[12px] leading-[1.4] text-[#71717a]";
const WHITE_CHIP =
	"flex items-center rounded-[8px] bg-white text-[#18181b] [box-shadow:0_0_0_1px_#e4e4e7]";

/**
 * A grid cell that enters with its group. `span` sets the columns and rows from a tablet up. On a phone
 * (two columns) a tile takes both, or one when `narrow`; `last` moves it to the end of a phone's list.
 */
function Tile({
	span,
	narrow = false,
	last = false,
	children,
}: {
	span: string;
	narrow?: boolean;
	last?: boolean;
	children: ReactNode;
}) {
	const phone = narrow ? "col-span-1" : "col-span-2";
	const tabletSpan =
		narrow || span.includes("min-[640px]:col-span-") ? "" : "min-[640px]:col-span-1";
	const order = last ? "order-last min-[640px]:order-none" : "";

	return <RevealItem className={`${phone} ${tabletSpan} ${order} ${span}`}>{children}</RevealItem>;
}

/** A small black notch: its flares, the real header and the content below it. */
function MiniNotch({ activePage, children }: { activePage: number; children: ReactNode }) {
	return (
		// The negative margin sits outside the zoomed notch, so it cancels the tile padding exactly.
		<div className="-mt-4 flex justify-center">
			<div
				className={`relative box-border flex w-[300px] flex-none flex-col gap-2.5 rounded-[0_0_22px_22px] bg-black p-[10px_16px_14px] text-left font-sans text-white ${ZOOM_MINI}`}
			>
				<Flares size={12} />
				<NotchHeader activePage={activePage} status={<BatteryRingIcon />} />
				{children}
			</div>
		</div>
	);
}

function Ring({
	percent,
	color,
	icon,
	label,
}: {
	percent: number;
	color: string;
	icon: ReactNode;
	label: string;
}) {
	const circumference = 2 * Math.PI * 15;

	return (
		<div className="flex flex-col items-center gap-1">
			<div className="relative size-[38px]">
				<svg
					aria-hidden="true"
					width="38"
					height="38"
					viewBox="0 0 38 38"
					className="absolute inset-0"
				>
					<circle cx="19" cy="19" r="15" fill="none" stroke="#e4e4e7" strokeWidth="3" />
					<circle
						cx="19"
						cy="19"
						r="15"
						fill="none"
						stroke={color}
						strokeWidth="3"
						strokeLinecap="round"
						strokeDasharray={`${(circumference * percent) / 100} ${circumference}`}
						transform="rotate(-90 19 19)"
					/>
				</svg>
				<div className="absolute inset-0 flex flex-col items-center justify-center leading-none">
					{icon}
					<span className="mt-0.5 text-[8px] font-medium">{percent}%</span>
				</div>
			</div>
			<span className="text-[9px] text-[#71717a]">{label}</span>
		</div>
	);
}

function MediaTile() {
	const t = useTranslations();

	return (
		<div
			className={`${PICTURE_TILE} ${TEXTURE.warm} min-h-[360px] bg-[position:50%_50%] p-0! min-[1000px]:min-h-0`}
		>
			<div className="flex w-full justify-center">
				<div className={ZOOM_MEDIA}>
					<MediaNotch />
				</div>
			</div>
			<div className="mt-auto flex flex-col gap-1 bg-gradient-to-t from-black/55 to-transparent p-6 pt-16">
				<div className="mb-2 flex items-center gap-2.5">
					{/* eslint-disable-next-line @next/next/no-img-element */}
					<img src="/logo.svg" width={30} height={30} alt="" className="rounded-lg" />
					<span className="text-[18px] font-semibold">Foldline</span>
				</div>
				<h3 className="m-0 text-[clamp(32px,3.4vw,50px)] leading-[1.05] font-semibold tracking-[-0.03em]">
					{t("hero.side1")}
				</h3>
				<p className="m-0 mt-1 max-w-[420px] text-[14px] leading-[1.45] text-white/85">
					{t("hero.side2")}
				</p>
			</div>
		</div>
	);
}

/** The short messages the pill gives, each with the icon it shows in the notch. */
function NoticeTile() {
	const t = useTranslations();
	const pills: Array<{ label: string; icon: ReactNode }> = [
		{ label: "Battery 80 %", icon: <BatteryRingIcon /> },
		{ label: "Volume 45", icon: <Volume2 size={13} color="#4c9aff" /> },
		{ label: "Brightness 70", icon: <Sun size={13} color="#ffcc00" /> },
		{ label: "Buds connected", icon: <Bluetooth size={13} color="#bf5af2" /> },
		{ label: "Caps Lock on", icon: <ArrowBigUp size={13} color="#ff9f0a" /> },
		{ label: t("word.muted"), icon: <VolumeX size={13} color="#ff453a" /> },
		{ label: "Download finished", icon: <Download size={13} color="#30d5c8" /> },
		{ label: t("word.rainIn"), icon: <CloudRain size={13} color="#64d2ff" /> },
	];

	return (
		<div className={LIGHT_TILE}>
			<div aria-hidden="true" className="flex flex-wrap items-start gap-1.5 min-[1000px]:flex-col">
				{pills.map(({ label, icon }) => (
					<span
						key={label}
						className="inline-flex h-[24px] items-center gap-1.5 rounded-full bg-black px-2.5 text-[11px] font-medium text-white"
					>
						{icon}
						{label}
					</span>
				))}
			</div>
			<div className="mt-auto flex flex-col gap-1 pt-3">
				<span className={`${EYEBROW} text-[#71717a]`}>{t("f.notice.label")}</span>
				<h3 className={TITLE}>{t("f.notice.a")}</h3>
			</div>
		</div>
	);
}

function FreeTile() {
	const t = useTranslations();

	return (
		<div className={LIGHT_TILE}>
			<div aria-hidden="true" className="relative h-[84px]">
				{/* eslint-disable-next-line @next/next/no-img-element */}
				<img
					src="/logo.svg"
					width={62}
					height={62}
					alt=""
					className="absolute top-0 left-1 -rotate-6 rounded-[16px] [box-shadow:0_8px_18px_-8px_rgba(37,99,235,0.6)]"
				/>
				<div className="absolute top-7 left-[52px] w-[92px] rotate-6 rounded-[6px] bg-white p-2 [box-shadow:0_8px_18px_-8px_rgba(15,23,42,0.4)]">
					<div className="flex items-center gap-1 text-[8px] font-semibold">
						{/* eslint-disable-next-line @next/next/no-img-element */}
						<img src="/logo.svg" width={9} height={9} alt="" className="rounded-[2px]" />
						Foldline
					</div>
					<div className="mt-3 font-mono text-[4px] leading-[1.4] text-[#71717a]">
						fold.hibatillah.com
						<br />
						FREE · OPEN SOURCE
					</div>
				</div>
			</div>
			<h3 className={`${TITLE} mt-auto pt-3`}>{t("bento.free.t")}</h3>
		</div>
	);
}

function ClipboardTile() {
	const t = useTranslations();

	return (
		<div className={LIGHT_TILE}>
			<span className={`${EYEBROW} text-[#71717a]`}>{t("g.keep.clipboard.t")}</span>
			<div aria-hidden="true" className="mt-2 flex flex-col gap-1">
				{[
					["fold.hibatillah.com", ""],
					["cargo test --workspace", "font-mono"],
					["Lunch at 12:30 on Friday?", ""],
				].map(([copy, font], index) => (
					<span
						key={copy}
						className={`${WHITE_CHIP} h-[19px] truncate px-2 text-[9px] ${font} ${index === 0 ? "" : "ml-2"}`}
					>
						{copy}
					</span>
				))}
			</div>
			<div className="mt-auto flex flex-col gap-0.5 pt-2">
				<h3 className={TITLE}>{t("bento.clipboard.t")}</h3>
				<p className={DESC}>{t("bento.clipboard.d")}</p>
			</div>
		</div>
	);
}

function ShelfTile() {
	const t = useTranslations();

	return (
		<div className={`${PICTURE_TILE} ${TEXTURE.blueWide} bg-[position:50%_50%]`}>
			<div aria-hidden="true" className="-mt-4 flex justify-center">
				<div className={`${ZOOM_SHELF} flex-none`}>
					<ShelfNotch />
				</div>
			</div>
			<div className="mt-auto flex flex-col gap-1 pt-3">
				<span className={`${EYEBROW} text-white/75`}>{t("strip.shelf.t")}</span>
				<h3 className={TITLE}>{t("strip.shelf.d")}</h3>
			</div>
		</div>
	);
}

function DevicesTile() {
	const t = useTranslations();

	return (
		<div className={LIGHT_TILE}>
			<span className={`${EYEBROW} text-[#71717a]`}>{t("g.keep.devices.t")}</span>
			<div aria-hidden="true" className="mt-2 flex justify-between">
				<Ring percent={80} color="#2f9e44" icon={<Headphones size={11} />} label="Earbuds" />
				<Ring percent={45} color="#f08c00" icon={<Mouse size={11} />} label="Mouse" />
				<Ring percent={18} color="#e03131" icon={<Keyboard size={11} />} label="Keyboard" />
			</div>
			<div className="mt-auto flex flex-col gap-0.5 pt-2">
				<h3 className={TITLE}>{t("g.keep.devices.t")}</h3>
				<p className={`${DESC} line-clamp-2`}>{t("g.keep.devices.d")}</p>
			</div>
		</div>
	);
}

/** A system stat box as the stats page draws it: label, value, a dotted baseline and two bars at its end. */
function StatBox({
	label,
	value,
	unit,
	light,
	dark,
	lightHeight,
	darkHeight,
}: {
	label: string;
	value: string;
	unit?: string;
	light: string;
	dark: string;
	lightHeight: number;
	darkHeight: number;
}) {
	return (
		<span
			className={`${WHITE_CHIP} relative h-[40px] flex-col items-start justify-start px-1.5 pt-1`}
		>
			<span className="text-[7px] leading-[1.3] text-[#71717a]">{label}</span>
			<span className="text-[12px] leading-[1.2] font-medium">
				{value}
				{unit && <span className="ml-0.5 text-[7px] font-normal text-[#71717a]">{unit}</span>}
			</span>
			<svg aria-hidden="true" viewBox="0 0 70 40" className="absolute inset-0 size-full">
				<line
					x1="6"
					x2="64"
					y1="33"
					y2="33"
					stroke="#a1a1aa"
					strokeWidth="1"
					strokeDasharray="1 3"
					strokeLinecap="round"
				/>
				<rect x="52" y={33 - lightHeight} width="3" height={lightHeight} rx="1" fill={light} />
				<rect x="57" y={33 - darkHeight} width="3" height={darkHeight} rx="1" fill={dark} />
			</svg>
		</span>
	);
}

function StatsTile() {
	const t = useTranslations();

	return (
		<div className={BLUE_TILE}>
			<span className={`${EYEBROW} text-[#71717a]`}>{t("g.keep.stats.t")}</span>
			<div aria-hidden="true" className="mt-2 grid grid-cols-2 gap-1">
				<StatBox
					label="CPU"
					value="12%"
					light="#a5d8ff"
					dark="#1c7ed6"
					lightHeight={5}
					darkHeight={8}
				/>
				<StatBox
					label="Memory"
					value="61%"
					light="#eebefa"
					dark="#ae3ec9"
					lightHeight={9}
					darkHeight={9}
				/>
				<StatBox
					label="GPU"
					value="8%"
					light="#ffd8a8"
					dark="#f08c00"
					lightHeight={3}
					darkHeight={4}
				/>
				<StatBox
					label="Network"
					value="2.4"
					unit="MB/s"
					light="#b2f2bb"
					dark="#2f9e44"
					lightHeight={6}
					darkHeight={10}
				/>
			</div>
			<p className={`${DESC} mt-auto line-clamp-2 pt-2`}>{t("strip.stats.d")}</p>
		</div>
	);
}

function KeysTile() {
	const t = useTranslations();

	return (
		<div className={LIGHT_TILE}>
			<div aria-hidden="true">
				<MiniNotch activePage={2}>
					<div className="flex items-center justify-between">
						<span className="leading-[1.3]">
							<span className="block text-[14px] font-semibold">Calendar</span>
							<span className="block text-[12px] text-white/70">Next: Design review, 10:00</span>
						</span>
						<span className="flex size-8 items-center justify-center rounded-[9px] bg-[#1d3a8a] text-[14px] font-medium text-[#6aa8ff]">
							23
						</span>
					</div>
				</MiniNotch>
			</div>
			<div aria-hidden="true" className="mt-3 flex gap-1.5">
				{["1", "2", "3"].map((key) => (
					<span
						key={key}
						className={`${WHITE_CHIP} size-7 justify-center text-[12px] font-medium ${key === "3" ? "text-primary" : ""}`}
					>
						{key}
					</span>
				))}
			</div>
			<div className="mt-auto flex flex-col gap-0.5 pt-2">
				<h3 className={TITLE}>{t("g.open.keys.t")}</h3>
				<p className={`${DESC} line-clamp-2`}>{t("g.open.keys.d")}</p>
			</div>
		</div>
	);
}

function PlanTile() {
	const t = useTranslations();

	return (
		<div className={LIGHT_TILE}>
			<span className={`${EYEBROW} text-[#71717a]`}>{t("f.plan.label")}</span>
			<div className="mt-2 flex min-h-0 flex-1 gap-4 max-[640px]:flex-col">
				<div className="flex w-[40%] flex-none flex-col max-[640px]:w-auto">
					<h3 className="m-0 text-[19px] leading-[1.15] font-medium tracking-[-0.02em]">
						{t("f.plan.a")}
					</h3>
					<p className={`${DESC} mt-auto first-letter:uppercase`}>{t("f.plan.b")}</p>
				</div>
				<div aria-hidden="true" className="flex min-w-0 flex-1 flex-col gap-1">
					<div className="grid grid-cols-7 gap-0.5 text-center text-[8px] text-[#71717a]">
						{["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"].map((day, index) => (
							<span
								key={day}
								className={`flex flex-col rounded-[5px] py-0.5 ${index === 0 ? "bg-primary text-white" : ""}`}
							>
								{day}
								<span className="text-[11px] font-medium">{23 + index}</span>
							</span>
						))}
					</div>
					{[
						["Design review", "10:00 to 10:45", "#1c7ed6"],
						["Focus session", "Session 1 of 4", "#f08c00"],
					].map(([title, sub, color]) => (
						<span key={title} className={`${WHITE_CHIP} gap-1.5 px-1.5 py-1 leading-[1.25]`}>
							<span
								className="h-5 w-0.5 flex-none rounded-full"
								style={{ backgroundColor: color }}
							/>
							<span className="min-w-0">
								<span className="block truncate text-[9px] font-medium">{title}</span>
								<span className="block truncate text-[8px] text-[#71717a]">{sub}</span>
							</span>
						</span>
					))}
				</div>
			</div>
		</div>
	);
}

function WeatherTile() {
	const t = useTranslations();

	return (
		<div className={WEATHER_TILE}>
			<span className={`${EYEBROW} text-white/75`}>{t("strip.weather.t")}</span>
			<h3 className={`${TITLE} mt-1`}>{t("strip.weather.d")}</h3>
			<div aria-hidden="true" className="mt-auto grid grid-cols-4 gap-1 pt-3">
				{[
					["Now", "24°", "0 %"],
					["+1 h", "25°", "30 %"],
					["+2 h", "23°", "80 %"],
					["+3 h", "22°", "60 %"],
				].map(([hour, degrees, rain]) => (
					<span
						key={hour}
						className="flex flex-col items-center rounded-[7px] bg-white/18 py-1 text-[8px] leading-[1.25]"
					>
						<span className="text-white/75">{hour}</span>
						<span className="text-[13px] font-semibold">{degrees}</span>
						<span className="text-white/75">{rain}</span>
					</span>
				))}
			</div>
		</div>
	);
}

function ExtendTile() {
	const t = useTranslations();

	return (
		<div className={`${PICTURE_TILE} ${TEXTURE.universtock} bg-[position:10%_80%]`}>
			<span className={`${EYEBROW} text-white/75`}>{t("own.label")}</span>
			<h3 className={`${TITLE} mt-1 text-[17px]`}>{t("own.a")}</h3>
			<pre
				aria-hidden="true"
				className="m-0 mt-auto overflow-hidden rounded-[10px] bg-black px-3 py-2 font-mono text-[9px] leading-[1.55] whitespace-pre-wrap text-white/90 max-[640px]:text-[10px]"
			>
				<span className="text-white/45">$ </span>scripts/new-content.sh page weather
				{"\n"}
				<span className="text-white/45"># every file compiles, with TODO lines to replace</span>
				{"\n"}
				<span className="text-white/45">$ </span>scripts/check-content-paths.sh --id weather
				{"\n"}
				<span className="text-white/45"># lists anything outside your files</span>
			</pre>
		</div>
	);
}

/** The Windows 11 taskbar of the tile: start, search and four app buttons, the notch's own among them. */
function Taskbar() {
	const dash = "mt-[3px] h-[2px] w-[5px] rounded-full bg-white/45";

	return (
		<div
			aria-hidden="true"
			className="absolute inset-x-0 bottom-0 flex h-[42px] items-center justify-center gap-2.5 bg-[#0d1736]/90 px-3 backdrop-blur-sm"
		>
			<svg width="20" height="20" viewBox="0 0 24 24" className="flex-none">
				<defs>
					<linearGradient id="start-gradient" x1="0" y1="0" x2="1" y2="1">
						<stop offset="0" stopColor="#4cc2ff" />
						<stop offset="1" stopColor="#0a78d4" />
					</linearGradient>
				</defs>
				<path d={windowsLogoPath} fill="url(#start-gradient)" />
			</svg>
			<span className="flex h-7 w-[104px] flex-none items-center gap-1.5 rounded-full bg-white/12 px-2.5 text-[11px] text-white [box-shadow:inset_0_0_0_1px_rgba(255,255,255,0.14)] max-[360px]:hidden">
				<Search size={12} />
				Search
			</span>
			<span className="flex flex-col items-center">
				<span className="flex size-[26px] items-center justify-center rounded-full bg-[#ececf1]">
					{/* eslint-disable-next-line @next/next/no-img-element */}
					<img
						src="/logos/warp.png"
						width={16}
						height={14}
						alt=""
						className="block h-[14px] w-auto"
					/>
				</span>
				<span className={dash} />
			</span>
			<span className="relative flex flex-col items-center">
				<span className="absolute bottom-full mb-2 rounded-[8px] bg-[#1b1b20] px-3 py-1.5 text-[11px] font-medium text-white whitespace-nowrap [box-shadow:0_6px_16px_-6px_rgba(0,0,0,0.6)]">
					Foldline
				</span>
				<span className="flex size-[28px] items-center justify-center rounded-[8px] bg-white/18 [box-shadow:inset_0_0_0_1px_rgba(255,255,255,0.22)]">
					{/* eslint-disable-next-line @next/next/no-img-element */}
					<img src="/logo.svg" width={22} height={22} alt="" className="rounded-[6px]" />
				</span>
				<span className="mt-[3px] h-[2px] w-3 rounded-full bg-[#4c9aff]" />
			</span>
			<span className="flex flex-col items-center">
				{/* eslint-disable-next-line @next/next/no-img-element */}
				<img
					src="/logos/claude.jpeg"
					width={26}
					height={26}
					alt=""
					className="block size-[26px] rounded-[7px]"
				/>
				<span className={dash} />
			</span>
			<span className="flex flex-col items-center">
				<span className="block size-[26px] overflow-hidden rounded-[7px]">
					{/* eslint-disable-next-line @next/next/no-img-element */}
					<img
						src="/logos/zed.jpeg"
						width={26}
						height={26}
						alt=""
						className="block size-full scale-[1.12]"
					/>
				</span>
				<span className={dash} />
			</span>
		</div>
	);
}

function WindowsTile() {
	const t = useTranslations();

	return (
		<div
			className={`${TILE_BASE} ${TILE_MIN_HEIGHT} ${TEXTURE.warm} min-h-[190px] bg-no-repeat bg-[position:30%_40%] bg-[length:340%_auto] [image-rendering:pixelated] text-white`}
		>
			<span className={`${EYEBROW} text-white/85`}>{t("bento.windows.label")}</span>
			<h3 className={`${TITLE} mt-1 text-[17px]`}>{t("bento.windows.t")}</h3>
			<Taskbar />
		</div>
	);
}

function ProofTile() {
	const t = useTranslations();
	const perf = t.raw("light.perf") as Messages["light"]["perf"];
	// The idle CPU reading, the start memory and the open and close time, in the order of the banner.
	const figures = [perf[1], perf[0], perf[2]];

	return (
		<div className={BLUE_TILE}>
			<span className={`${EYEBROW} text-[#71717a]`}>{t("proof.a")}</span>
			<div className="mt-auto grid grid-cols-3 gap-3 pb-3">
				{figures.map((figure) => {
					const [value, unit] = figure.b.split(" ");

					return (
						<div key={figure.a} className="flex min-w-0 flex-col">
							<span className="text-[clamp(26px,3vw,40px)] leading-none font-medium tracking-[-0.03em] text-primary tabular-nums">
								{value}
								<span className="ml-0.5 text-[0.4em] font-normal text-[#52525b]">{unit}</span>
							</span>
							<span className={`${DESC} mt-1.5`}>{figure.a}</span>
						</div>
					);
				})}
			</div>
			<span
				aria-hidden="true"
				className="absolute inset-x-4 bottom-4 h-[2px] rounded-full bg-primary/70"
			/>
		</div>
	);
}

/** The features of Foldline as one grid of small drawings; each tile reuses text the site already has. */
export function Bento() {
	const t = useTranslations();

	return (
		<section
			id="features"
			className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] pt-[clamp(64px,_7vw,_96px)]"
		>
			<Reveal className="mb-10 max-[640px]:mb-8">
				<TwoTone first={t("all.a")} second={t("all.b")} />
			</Reveal>
			<RevealGroup className="grid grid-flow-dense grid-cols-2 gap-2.5 min-[640px]:grid-cols-2 min-[1000px]:auto-rows-[176px] min-[1000px]:grid-cols-6">
				<Tile span="min-[640px]:col-span-2 min-[1000px]:col-span-3 min-[1000px]:row-span-2">
					<MediaTile />
				</Tile>
				<Tile span="min-[1000px]:row-span-2">
					<NoticeTile />
				</Tile>
				<Tile span="" narrow>
					<FreeTile />
				</Tile>
				<Tile span="">
					<ClipboardTile />
				</Tile>
				<Tile span="min-[640px]:col-span-2">
					<ShelfTile />
				</Tile>
				<Tile span="" narrow>
					<DevicesTile />
				</Tile>
				<Tile span="">
					<StatsTile />
				</Tile>
				<Tile span="">
					<KeysTile />
				</Tile>
				<Tile span="min-[640px]:col-span-2">
					<PlanTile />
				</Tile>
				<Tile span="min-[640px]:col-span-2 min-[1000px]:col-span-1">
					<WeatherTile />
				</Tile>
				<Tile span="min-[640px]:col-span-2">
					<ExtendTile />
				</Tile>
				<Tile span="min-[640px]:col-span-2" last>
					<WindowsTile />
				</Tile>
				<Tile span="min-[640px]:col-span-2">
					<ProofTile />
				</Tile>
			</RevealGroup>
		</section>
	);
}
