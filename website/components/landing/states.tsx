import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { DefaultStatus, Flares, NotchHeader } from "@/components/notch/open-notch";
import { ClockPill, VolumePill } from "@/components/notch/pill";
import { PAGE_PICTURE } from "@/lib/pages";

import { CtaLink } from "./cta-link";
import { Reveal, RevealGroup, RevealItem } from "./reveal";
import { PANEL, TEXTURE, TEXTURE_FILL } from "./textures";
import { TwoTone } from "./two-tone";

const CARD = "flex flex-col gap-4";
const SUBTITLE = "m-0 text-[clamp(20px,1.8vw,24px)] font-medium tracking-[-0.02em]";

/** A dithered panel the notch hangs from, like the top edge of the screen. */
function NotchPanel({ children }: { children: ReactNode }) {
	return (
		<div className={`${PANEL} ${TEXTURE.universtock} ${TEXTURE_FILL} min-h-[104px] items-start`}>
			<div className="flex flex-col items-center gap-3">{children}</div>
		</div>
	);
}

/** The top of the open notch: its header, where the marks of activities sit at the right. */
function OpenHeader() {
	return (
		<div className="relative box-border w-[320px] max-w-full rounded-[0_0_22px_22px] bg-black px-4 pt-[10px] pb-[14px] font-sans text-white">
			<Flares size={12} />
			<NotchHeader activePage={0} status={<DefaultStatus />} />
		</div>
	);
}

function Caption({ title, text }: { title: string; text: string }) {
	return (
		<div>
			<h4 className="m-0 text-[17px] font-medium">{title}</h4>
			<p className="mt-1.5 text-[15px] leading-[1.55] text-muted-foreground">{text}</p>
		</div>
	);
}

/** The two things the notch shows: a page when it is open, and activities in both states. */
export function States() {
	const t = useTranslations();

	return (
		<section
			id="states"
			className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] pt-[clamp(120px,13vw,200px)] max-[640px]:pt-24"
		>
			<Reveal className="mb-[56px] grid grid-cols-[repeat(2,minmax(0,1fr))] gap-x-12 gap-y-6 max-[900px]:grid-cols-[minmax(0,1fr)] max-[640px]:mb-8 max-[640px]:gap-4">
				<p className="m-0 text-[clamp(24px,2.2vw,30px)] font-medium tracking-[-0.02em]">
					<span className="text-muted-foreground">02</span>
					{"  "}
					{t("states.label")}
				</p>
				<div>
					<TwoTone first={t("states.a")} second={t("states.b")} />
					<p className="mt-9 max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground max-[640px]:mt-5">
						{t("states.p")}
					</p>
				</div>
			</Reveal>

			<Reveal className="grid grid-cols-[repeat(2,minmax(0,1fr))] items-center gap-x-12 gap-y-8 max-[900px]:grid-cols-[minmax(0,1fr)]">
				<div>
					<h3 className={SUBTITLE}>{t("states.page.t")}</h3>
					<p className="mt-3 max-w-[56ch] text-[17px] leading-[1.6] text-muted-foreground">
						{t("states.page.d")}
					</p>
				</div>
				{/* eslint-disable-next-line @next/next/no-img-element */}
				<img
					src="/pages/media.jpg"
					width={PAGE_PICTURE.media.width}
					height={PAGE_PICTURE.media.height}
					alt={t("states.page.alt")}
					loading="lazy"
					decoding="async"
					className="block h-auto w-full rounded-[14px]"
				/>
			</Reveal>

			<div className="mt-20 border-t border-border pt-10">
				<Reveal className="mb-8 max-w-[620px]">
					<h3 className={SUBTITLE}>{t("states.activity.t")}</h3>
					<p className="mt-3 text-[17px] leading-[1.6] text-muted-foreground">
						{t("states.activity.d")}
					</p>
				</Reveal>
				<RevealGroup className="grid grid-cols-[repeat(3,minmax(0,1fr))] gap-x-6 gap-y-10 max-[900px]:grid-cols-[minmax(0,1fr)]">
					<RevealItem className={CARD}>
						<NotchPanel>
							<OpenHeader />
						</NotchPanel>
						<Caption title={t("states.open.t")} text={t("states.open.d")} />
					</RevealItem>
					<RevealItem className={CARD}>
						<NotchPanel>
							<ClockPill />
						</NotchPanel>
						<Caption title={t("states.collapsed.t")} text={t("states.collapsed.d")} />
					</RevealItem>
					<RevealItem className={CARD}>
						<NotchPanel>
							<VolumePill />
						</NotchPanel>
						<Caption title={t("states.demand.t")} text={t("states.demand.d")} />
					</RevealItem>
				</RevealGroup>
			</div>

			<Reveal className="mt-12">
				<CtaLink href="/#pages" size="lg" variant="muted">
					{t("states.more")}
					<span aria-hidden="true">→</span>
				</CtaLink>
			</Reveal>
		</section>
	);
}
