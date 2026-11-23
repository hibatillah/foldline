import { MediaNotch } from "@/components/notch/media";

import { NotchFrame } from "./notch-frame";

/** The media notch shown beside the hero text. */
export function HeroFrame() {
	return (
		<NotchFrame variant="hero" seed={1} zoom={1.08} notch="media" texture="blueWide">
			<MediaNotch />
		</NotchFrame>
	);
}
