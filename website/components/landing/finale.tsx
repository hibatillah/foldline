import { useTranslations } from "next-intl";
import { CtaLink } from "./cta-link";
import { WindowsIcon } from "./windows-icon";

export function Finale() {
	const t = useTranslations();

	return (
		<section className="mx-auto box-border max-w-[1280px] pt-[clamp(120px,_13vw,_180px)] px-[clamp(16px,5vw,80px)]">
			<div className="relative overflow-hidden rounded-[20px] [background:url(/textures/blue-dither-wide.png)_center_/_cover_no-repeat,_#1c5fe0] p-[clamp(64px,_7vw,_96px)_24px] text-center flex flex-col items-center gap-[28px]">
				<div
					aria-hidden="true"
					className="absolute inset-0 [background:linear-gradient(180deg,_rgba(3,10,40,0.35)_0%,_rgba(3,10,40,0.2)_100%)] pointer-events-none"
				/>
				<div
					aria-hidden="true"
					className="absolute left-[50%] top-0 [transform:translateX(-50%)] w-[150px] h-[33px] bg-[#000] rounded-[0_0_12px_12px]"
				>
					<span
						aria-hidden="true"
						className="absolute left-[-6px] top-0 w-[6px] h-[6px] [background:radial-gradient(circle_6px_at_0_100%,_transparent_98%,_#000_100%)]"
					/>
					<span
						aria-hidden="true"
						className="absolute right-[-6px] top-0 w-[6px] h-[6px] [background:radial-gradient(circle_6px_at_100%_100%,_transparent_98%,_#000_100%)]"
					/>
				</div>
				<h2 className="text-[clamp(26px,2.4vw,32px)] leading-[1.22] relative m-0 font-medium tracking-[-0.025em] text-[#fff]">
					{t("final.a")}
					<br />
					<span className="text-[rgba(255,255,255,0.72)]">{t("final.b")}</span>
				</h2>
				<div className="relative flex flex-wrap justify-center gap-[12px]">
					<CtaLink href="/download" size="lg" variant="light">
						<WindowsIcon />
						{t("hero.cta")}
					</CtaLink>
					<CtaLink href="/releases" size="lg" variant="lightOutline">
						{t("final.notes")}
					</CtaLink>
				</div>
			</div>
		</section>
	);
}
