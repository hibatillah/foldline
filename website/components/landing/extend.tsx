import { useTranslations } from "next-intl";

import { CountdownNotch } from "@/components/notch/countdown";
import { PHONE_ZOOM_CLASS } from "@/components/notch/sizes";
import type en from "@/messages/en.json";

import { CtaLink } from "./cta-link";
import { ExternalArrow } from "./external-arrow";
import { PANEL, TEXTURE, TEXTURE_FILL } from "./textures";

type Messages = typeof en;

export function Extend() {
	const t = useTranslations();
	const ownSteps = t.raw("own.steps") as Messages["own"]["steps"];

	return (
		<section
			id="extend"
			className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] pt-[clamp(120px,13vw,200px)] max-[640px]:pt-24"
		>
			<div className="mb-[72px] grid grid-cols-[repeat(2,minmax(0,1fr))] gap-x-12 gap-y-6 max-[900px]:grid-cols-[minmax(0,1fr)] max-[640px]:mb-10 max-[640px]:gap-4">
				<p className="m-0 text-[clamp(24px,2.2vw,30px)] font-medium tracking-[-0.02em]">
					<span className="text-muted-foreground">06</span>
					{"  "}
					{t("own.label")}
				</p>
				<div>
					<h2 className="m-0 text-[clamp(26px,2.4vw,32px)] leading-[1.22] font-medium tracking-[-0.025em] text-balance">
						{t("own.a")}
						<br />
						<span className="text-muted-foreground">{t("own.b")}</span>
					</h2>
					<p className="mt-9 max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground max-[640px]:mt-5">
						{t("own.p")}
					</p>
					<div className="mt-7">
						<CtaLink
							href="https://github.com/hibatillah/foldline/blob/master/CONTRIBUTING.md#add-your-own-content"
							size="lg"
							variant="muted"
						>
							{t("own.guide")}
							<ExternalArrow />
						</CtaLink>
					</div>
				</div>
			</div>
			<div className="grid grid-cols-[repeat(2,minmax(0,1fr))] items-stretch gap-6 max-[900px]:grid-cols-[minmax(0,1fr)]">
				<div
					className={`${PANEL} ${TEXTURE.universtock} ${TEXTURE_FILL} min-h-[300px] bg-[position:30%_60%]`}
				>
					<div
						className={`relative [zoom:1.1] max-[900px]:[zoom:0.75]! ${PHONE_ZOOM_CLASS.countdown}`}
					>
						<CountdownNotch />
					</div>
				</div>
				<div
					className={`${PANEL} ${TEXTURE.henrikLight} ${TEXTURE_FILL} min-h-[300px] items-center bg-[position:50%_40%] px-4 py-[clamp(24px,4vw,40px)]`}
				>
					<pre className="m-0 box-border w-[min(520px,100%)] overflow-x-auto rounded-xl bg-[#0b0b0c] p-[22px_24px] font-mono text-[14px] leading-[1.9] whitespace-pre-wrap text-[#e4e4e7] max-[640px]:p-[18px] max-[640px]:text-[12.5px] max-[640px]:whitespace-pre">
						<span className="text-[#8e8e99]">{"$"}</span>
						{" scripts/new-content.sh page countdown\n"}
						<span className="text-[#8e8e99]">{"›"}</span>
						{" /add-content a page that counts down to a date\n"}
						<span className="text-[#6e6e7a]">{"  # in Claude Code\n"}</span>
						<span className="text-[#8e8e99]">{"$"}</span>
						{" scripts/check-content-paths.sh --id countdown"}
					</pre>
				</div>
			</div>
			<div className="mt-12 grid grid-cols-[repeat(3,minmax(0,1fr))] gap-8 max-[900px]:grid-cols-[minmax(0,1fr)]">
				{ownSteps.map((step) => (
					<div className="flex flex-col gap-1.5 border-t border-border pt-[18px]" key={step.t}>
						<span className="text-[18px] font-medium">{step.t}</span>
						<span className="text-[15px] leading-[1.55] text-muted-foreground">{step.d}</span>
					</div>
				))}
			</div>
		</section>
	);
}
