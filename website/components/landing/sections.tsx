import { CalendarWeekNotch } from "@/components/notch/calendar-week";
import { ClipboardNotch } from "@/components/notch/clipboard";
import { DownloadBanner } from "@/components/notch/download";
import { MediaNotch } from "@/components/notch/media";
import { WhatsAppBanner } from "@/components/notch/notification";
import { PrayerNotch } from "@/components/notch/prayer";
import { ShelfNotch } from "@/components/notch/shelf";
import { WeatherNotch } from "@/components/notch/weather";

import { BatteryRings } from "./elements/battery-rings";
import { DevicesGrid } from "./elements/devices-grid";
import { FocusTimer } from "./elements/focus-timer";
import { NumberKeys } from "./elements/number-keys";
import { PinStates } from "./elements/pin-states";
import { RainCard } from "./elements/rain-card";
import { StatsGrid } from "./elements/stats-grid";
import { StatusTiles } from "./elements/status-tiles";
import { VolumeSliders } from "./elements/volume-sliders";
import { FeatureGrid, type FeatureItem } from "./feature-grid";
import { FeatureHead, type FeatureKey } from "./feature-head";

function FeatureSection({
	section,
	number,
	items,
}: {
	section: FeatureKey;
	number: string;
	items: FeatureItem[];
}) {
	return (
		<section className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] pt-[clamp(120px,13vw,200px)] max-[640px]:pt-24">
			<FeatureHead section={section} number={number} />
			<FeatureGrid section={section} items={items} />
		</section>
	);
}

export function SectionOpen() {
	return (
		<FeatureSection
			section="open"
			number="01"
			items={[
				{
					key: "media",
					span: 3,
					kind: "frame",
					visual: <MediaNotch />,
					notch: "media",
					zoom: 1.3,
					seed: 2,
					texture: "henrikLight",
				},
				{ key: "keys", span: 1, kind: "card", visual: <NumberKeys /> },
				{ key: "pin", span: 1, kind: "card", visual: <PinStates /> },
				{
					key: "calendar",
					span: 3,
					kind: "frame",
					visual: <CalendarWeekNotch />,
					notch: "calendar",
					zoom: 1.3,
					seed: 8,
					texture: "henrikDark",
				},
			]}
		/>
	);
}

export function SectionNotice() {
	return (
		<FeatureSection
			section="notice"
			number="02"
			items={[
				{
					key: "notification",
					span: 2,
					kind: "frame",
					visual: <WhatsAppBanner />,
					notch: "notification",
					zoom: 1.2,
					seed: 3,
					texture: "universtock",
				},
				{
					key: "download",
					span: 2,
					kind: "frame",
					visual: <DownloadBanner />,
					notch: "download",
					zoom: 1.3,
					seed: 9,
					texture: "henrikLight",
				},
				{ key: "volume", span: 1, kind: "card", visual: <VolumeSliders />, zoom: 1.05 },
				{ key: "rings", span: 1, kind: "card", visual: <BatteryRings />, zoom: 1.05 },
				{ key: "indicators", span: 2, kind: "card", visual: <StatusTiles />, zoom: 1.05 },
			]}
		/>
	);
}

export function SectionPlan() {
	return (
		<FeatureSection
			section="plan"
			number="03"
			items={[
				{ key: "rain", span: 1, kind: "card", visual: <RainCard /> },
				{
					key: "weather",
					span: 3,
					kind: "frame",
					visual: <WeatherNotch />,
					notch: "weather",
					zoom: 1.5,
					seed: 4,
					texture: "universtock",
				},
				{
					key: "prayer",
					span: 3,
					kind: "frame",
					visual: <PrayerNotch />,
					notch: "prayer",
					zoom: 1.45,
					seed: 10,
					texture: "henrikLight",
				},
				{ key: "timer", span: 1, kind: "card", visual: <FocusTimer />, zoom: 0.82 },
			]}
		/>
	);
}

export function SectionKeep() {
	return (
		<FeatureSection
			section="keep"
			number="04"
			items={[
				{
					key: "shelf",
					span: 3,
					kind: "frame",
					visual: <ShelfNotch />,
					notch: "shelf",
					zoom: 1.2,
					seed: 5,
					texture: "universtock",
				},
				{ key: "devices", span: 1, kind: "card", visual: <DevicesGrid />, zoom: 1 },
				{ key: "stats", span: 1, kind: "card", visual: <StatsGrid />, zoom: 0.82 },
				{
					key: "clipboard",
					span: 3,
					kind: "frame",
					visual: <ClipboardNotch />,
					notch: "clipboard",
					zoom: 1.35,
					seed: 11,
					texture: "henrikDark",
				},
			]}
		/>
	);
}
