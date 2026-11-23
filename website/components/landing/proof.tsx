import { useTranslations } from "next-intl";
import { ExternalLink } from "./external-link";

export function Proof() {
	const t = useTranslations();

	return (
		<>
			<div className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] mt-[clamp(80px,_9vw,_120px)]">
				<hr className="m-0 border-0 border-t border-border" />
			</div>
			<section className="mx-auto box-border max-w-[1280px] p-[clamp(32px,_3.5vw,_48px)_clamp(16px,_5vw,_80px)] flex flex-col gap-[32px] items-start">
				<h2 className="text-[clamp(26px,2.4vw,32px)] leading-[1.22] m-0 font-medium tracking-[-0.025em] text-balance">
					{t("proof.a")}
					<br />
					<span className="text-muted-foreground">{t("proof.b")}</span>
				</h2>
				<div className="flex flex-wrap items-center gap-y-[12px] gap-x-[24px]">
					<ExternalLink
						href="https://github.com/hibatillah/foldline/blob/master/docs/reference/performance-baseline.md"
						className="flex items-center gap-[14px] no-underline text-[15px] font-medium"
					>
						<span className="w-[40px] h-[40px] rounded-full bg-muted flex items-center justify-center">
							<svg
								aria-hidden="true"
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								strokeWidth="2"
								strokeLinecap="round"
							>
								<path d="M7 17 17 7M8 7h9v9" />
							</svg>
						</span>
						{t("proof.link")}
					</ExternalLink>
					<span className="text-[14px] text-muted-foreground">{t("proof.source")}</span>
				</div>
			</section>
			<div className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)]">
				<hr className="m-0 border-0 border-t border-border" />
			</div>
		</>
	);
}
