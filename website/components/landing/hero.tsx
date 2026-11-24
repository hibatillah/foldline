import { useTranslations } from "next-intl";
import type en from "@/messages/en.json";
import { CtaLink } from "./cta-link";
import { HeroFrame } from "./hero-frame";
import { Reveal } from "./reveal";
import { WindowsIcon } from "./windows-icon";

type Messages = typeof en;

export function Hero() {
	const t = useTranslations();
	const heroList = t.raw("heroList") as Messages["heroList"];

	return (
		<section className="mx-auto box-border max-w-[1280px] pt-[clamp(48px,_5vw,_64px)] px-[clamp(16px,5vw,80px)]">
			<Reveal>
				<h1
					lang="en"
					className="text-[clamp(30px,3vw,40px)] leading-[1.18] m-0 font-medium tracking-[-0.03em]"
				>
					{t("hero.a")}
					<br />
					<span className="text-muted-foreground">{t("hero.b")}</span>
				</h1>
			</Reveal>
			<Reveal delay={0.1} className="mt-[44px]">
				<div className="flex flex-wrap items-center gap-[12px]">
					<CtaLink href="/download" size="lg" variant="primary">
						<WindowsIcon />
						{t("hero.cta")}
					</CtaLink>
					<CtaLink href="/#extend" size="lg" variant="muted">
						{t("hero.own")}
					</CtaLink>
				</div>
			</Reveal>
			<Reveal delay={0.2} className="mt-[64px] max-[640px]:mt-10">
				<div className="grid grid-cols-[minmax(0,7fr)_minmax(0,3fr)] items-stretch gap-12 max-[900px]:grid-cols-[minmax(0,1fr)] max-[640px]:gap-8">
					<div className="flex">
						<HeroFrame />
					</div>
					<div className="flex flex-col justify-between gap-[32px] p-[4px_0]">
						<h2 className="text-[clamp(22px,2vw,28px)] leading-[1.25] m-0 font-medium tracking-[-0.025em] text-balance">
							{t("hero.side1")}
							<br />
							<span className="text-muted-foreground">{t("hero.side2")}</span>
						</h2>
						<div className="flex flex-col gap-[12px]">
							<p className="m-0 flex flex-wrap gap-y-[6px] text-[16px] font-medium leading-[1.5]">
								{heroList.map((item, index) => (
									<span key={item}>
										{item}
										{index < heroList.length - 1 && (
											<span className="text-primary p-[0_10px]">·</span>
										)}
									</span>
								))}
							</p>
						</div>
					</div>
				</div>
			</Reveal>
		</section>
	);
}
