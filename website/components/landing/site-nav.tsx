import Image from "next/image";
import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";

import { CtaLink } from "./cta-link";
import { GithubIcon } from "./github-icon";
import { LanguageSwitch } from "./language-switch";
import { StickyHeader } from "./sticky-header";

/** The header of every page; its section links lead to the landing page from any other page. */
export function SiteNav() {
	const t = useTranslations();

	return (
		<StickyHeader>
			<nav
				aria-label={t("navAria")}
				className="mx-auto box-border grid h-[60px] max-w-[1280px] grid-cols-[1fr_auto_1fr] items-center gap-6 max-[1000px]:grid-cols-[1fr_auto] rounded-2xl bg-white/0 px-[clamp(16px,5vw,80px)] backdrop-blur-0 backdrop-saturate-[1.8] [box-shadow:0_0_0_1px_transparent,0_12px_32px_-18px_transparent] transition-[max-width,translate,padding,background-color,box-shadow,backdrop-filter] duration-250 group-data-[stuck=true]:max-w-[1240px] group-data-[stuck=true]:translate-y-[14px] group-data-[stuck=true]:bg-[rgba(255,255,255,0.4)] group-data-[stuck=true]:pr-3 group-data-[stuck=true]:pl-5 group-data-[stuck=true]:backdrop-blur-[32px] group-data-[stuck=true]:[box-shadow:0_0_0_1px_color-mix(in_oklch,var(--foreground)_8%,transparent),0_12px_32px_-18px_rgba(15,23,42,0.28)] motion-reduce:transition-none"
			>
				<Link
					href="/"
					className="flex w-fit items-center justify-self-start no-underline font-semibold text-[18px] tracking-[-0.01em]"
				>
					<Image src="/logo.svg" alt="" width={26} height={26} className="block rounded-[7px]" />
					{/* The name shows only once the header floats. */}
					<span className="max-w-0 overflow-hidden pl-0 opacity-0 transition-[max-width,padding,opacity] duration-250 group-data-[stuck=true]:max-w-[120px] group-data-[stuck=true]:pl-[10px] group-data-[stuck=true]:opacity-100 motion-reduce:transition-none">
						Foldline
					</span>
				</Link>
				<div className="flex max-[1000px]:hidden gap-[30px] text-[15px]">
					<Link href="/#features" className="no-underline text-foreground transition-colors">
						{t("nav.features")}
					</Link>
					<Link href="/#pages" className="no-underline text-foreground transition-colors">
						{t("nav.pages")}
					</Link>
					<Link href="/#faq" className="no-underline text-foreground transition-colors">
						{t("nav.faq")}
					</Link>
					<Link href="/releases" className="no-underline text-foreground transition-colors">
						{t("nav.releases")}
					</Link>
				</div>
				<div className="flex items-center justify-end gap-[16px]">
					<LanguageSwitch />
					<div className="flex items-center gap-[8px]">
						<CtaLink
							className="max-[640px]:hidden"
							href="https://github.com/hibatillah/foldline"
							size="sm"
							variant="muted"
						>
							<GithubIcon />
							GitHub
						</CtaLink>
						<CtaLink href="/download" size="sm" variant="primary">
							{t("nav.cta")}
						</CtaLink>
					</div>
				</div>
			</nav>
		</StickyHeader>
	);
}
