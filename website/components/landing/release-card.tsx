import Image from "next/image";
import { useTranslations } from "next-intl";

export function ReleaseCard() {
	const t = useTranslations();

	return (
		<div className="w-[min(480px,_calc(100%_-_32px))] bg-card rounded-[12px] [box-shadow:0_0_0_1px_color-mix(in_oklch,_var(--foreground)_9%,_transparent),_0_18px_40px_-22px_rgba(15,23,42,0.35)] text-left text-[14px]">
			<div className="flex items-center gap-[10px] p-[16px_18px] [border-bottom:1px_solid_var(--border)]">
				<Image src="/logo.svg" alt="" width={22} height={22} className="rounded-[6px]" />
				<span className="font-semibold">Foldline v0.6.0</span>
				<span className="ml-[auto] text-[12px] font-medium text-primary">
					{t("install.latest")}
				</span>
			</div>
			<div className="p-[6px_18px_8px]">
				<div className="flex items-center gap-[12px] p-[12px_0] [border-bottom:1px_solid_var(--border)]">
					<svg
						aria-hidden="true"
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="var(--muted-foreground)"
						strokeWidth="2"
						strokeLinecap="round"
						strokeLinejoin="round"
					>
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
					</svg>
					<span className="flex flex-col min-w-0">
						<span className="font-mono text-[13px] whitespace-nowrap overflow-hidden text-ellipsis">
							foldline-v0.6.0-windows-x64-setup.exe
						</span>
						<span className="text-[12px] text-muted-foreground">{t("install.installer")}</span>
					</span>
					<span className="ml-[auto] text-[12px] text-muted-foreground whitespace-nowrap tabular-nums">
						5.75 MB
					</span>
				</div>
				<div className="flex items-center gap-[12px] p-[12px_0] [border-bottom:1px_solid_var(--border)]">
					<svg
						aria-hidden="true"
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="var(--muted-foreground)"
						strokeWidth="2"
						strokeLinecap="round"
						strokeLinejoin="round"
					>
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
					</svg>
					<span className="flex flex-col min-w-0">
						<span className="font-mono text-[13px] whitespace-nowrap overflow-hidden text-ellipsis">
							foldline.exe
						</span>
						<span className="text-[12px] text-muted-foreground">{t("install.portable")}</span>
					</span>
					<span className="ml-[auto] text-[12px] text-muted-foreground whitespace-nowrap tabular-nums" />
				</div>
			</div>
			<div className="p-[10px_18px_16px] text-[13px] text-muted-foreground">{t("install.req")}</div>
		</div>
	);
}
