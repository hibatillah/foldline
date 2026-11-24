import { useTranslations } from "next-intl";
import type en from "@/messages/en.json";

type Messages = typeof en;

export function Light() {
	const t = useTranslations();
	const lightPriv = t.raw("light.priv") as Messages["light"]["priv"];
	const lightPerf = t.raw("light.perf") as Messages["light"]["perf"];

	return (
		<section
			id="light"
			className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] pt-[clamp(120px,13vw,200px)] max-[640px]:pt-24"
		>
			<div className="grid grid-cols-[repeat(2,minmax(0,1fr))] gap-x-12 gap-y-6 mb-[72px] max-[900px]:grid-cols-[minmax(0,1fr)] max-[640px]:mb-10 max-[640px]:gap-4">
				<p className="text-[clamp(24px,2.2vw,30px)] font-medium tracking-[-0.02em] m-0">
					<span className="text-muted-foreground">02</span>
					{"\u00a0\u00a0"}
					{t("light.label")}
				</p>
				<div>
					<h2 className="text-[clamp(26px,2.4vw,32px)] leading-[1.22] m-0 font-medium tracking-[-0.025em] text-balance">
						{t("light.a")}
						<br />
						<span className="text-muted-foreground">{t("light.b")}</span>
					</h2>
					<p className="max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground mt-9 max-[640px]:mt-5">
						{t("light.p")}
					</p>
				</div>
			</div>
			<div className="grid grid-cols-[repeat(2,minmax(0,1fr))] gap-12 max-[900px]:grid-cols-[minmax(0,1fr)] max-[900px]:[&>div:nth-child(2)]:order-first">
				<div>
					<p className="m-[0_0_14px] text-[15px] font-medium flex items-center gap-[8px]">
						<span className="w-[8px] h-[8px] rounded-[2px] bg-primary" />
						{t("light.privTitle")}
					</p>
					<table className="w-full [border-collapse:collapse] text-[15px] max-[640px]:[&_td]:block max-[640px]:[&_td]:pt-0! max-[640px]:[&_td]:pr-0! max-[640px]:[&_td]:pb-3.5! max-[640px]:[&_td]:pl-[25px]! max-[640px]:[&_td]:text-left! max-[640px]:[&_th]:block max-[640px]:[&_th]:px-0! max-[640px]:[&_th]:pt-3.5! max-[640px]:[&_th]:pb-1!">
						<tbody>
							{lightPriv.map((pv) => (
								<tr className="[border-top:1px_solid_var(--border)]" key={pv.a}>
									<th
										scope="row"
										className="text-left font-normal p-[16px_16px_16px_0] text-muted-foreground"
									>
										<span className="inline-flex items-center gap-[10px]">
											<svg
												aria-hidden="true"
												width="15"
												height="15"
												viewBox="0 0 24 24"
												fill="none"
												stroke="var(--primary)"
												strokeWidth="2"
												strokeLinecap="round"
												strokeLinejoin="round"
											>
												<path d="M7 7h10v10" />
												<path d="M7 17 17 7" />
											</svg>
											{pv.a}
										</span>
									</th>
									<td className="text-right p-[16px_0]">{pv.b}</td>
								</tr>
							))}
						</tbody>
					</table>
					<p className="m-0 pt-[14px] [border-top:1px_solid_var(--border)] text-[13px] leading-[1.5] text-muted-foreground">
						{t("light.privNote")}
					</p>
				</div>
				<div>
					<p className="m-[0_0_14px] text-[15px] font-medium flex items-center gap-[8px]">
						<span className="w-[8px] h-[8px] rounded-[2px] bg-primary" />
						{t("light.perfTitle")}
					</p>
					<table className="w-full [border-collapse:collapse] text-[15px]">
						<tbody>
							{lightPerf.map((pf) => (
								<tr className="[border-top:1px_solid_var(--border)]" key={pf.a}>
									<th
										scope="row"
										className="text-left font-normal p-[16px_0] text-muted-foreground"
									>
										{pf.a}
									</th>
									<td className="text-right p-[16px_0] text-[16px] font-medium tracking-[-0.01em] tabular-nums text-primary">
										{pf.b}
									</td>
								</tr>
							))}
						</tbody>
					</table>
					<p className="m-0 pt-[14px] [border-top:1px_solid_var(--border)] text-[13px] leading-[1.5] text-muted-foreground">
						{t("light.note")}
					</p>
				</div>
			</div>
		</section>
	);
}
