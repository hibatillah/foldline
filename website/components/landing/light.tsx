import { useTranslations } from "next-intl";
import type en from "@/messages/en.json";

type Messages = typeof en;

export function Light() {
	const t = useTranslations();
	const lightPriv = t.raw("light.priv") as Messages["light"]["priv"];
	const lightPerf = t.raw("light.perf") as Messages["light"]["perf"];

	return (
		<section className="wrap feature">
			<div className="fhead">
				<p className="label" style={{ margin: "0" }}>
					<span style={{ color: "var(--muted-foreground)" }}>05</span>
					{"\u00a0\u00a0"}
					{t("light.label")}
				</p>
				<div>
					<h2
						className="h2"
						style={{
							margin: "0",
							fontWeight: "500",
							letterSpacing: "-0.025em",
							textWrap: "balance",
						}}
					>
						{t("light.a")}
						<br />
						<span style={{ color: "var(--muted-foreground)" }}>{t("light.b")}</span>
					</h2>
					<p className="body" style={{ margin: "36px 0 0" }}>
						{t("light.p")}
					</p>
				</div>
			</div>
			<div className="duo">
				<div>
					<p
						style={{
							margin: "0 0 14px",
							fontSize: "15px",
							fontWeight: "500",
							display: "flex",
							alignItems: "center",
							gap: "8px",
						}}
					>
						<span
							style={{
								width: "8px",
								height: "8px",
								borderRadius: "2px",
								background: "var(--primary)",
							}}
						/>
						{t("light.privTitle")}
					</p>
					<table
						className="priv"
						style={{ width: "100%", borderCollapse: "collapse", fontSize: "15px" }}
					>
						<tbody>
							{lightPriv.map((pv) => (
								<tr style={{ borderTop: "1px solid var(--border)" }} key={pv.a}>
									<th
										scope="row"
										style={{
											textAlign: "left",
											fontWeight: "400",
											padding: "16px 16px 16px 0",
											color: "var(--muted-foreground)",
										}}
									>
										<span style={{ display: "inline-flex", alignItems: "center", gap: "10px" }}>
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
									<td style={{ textAlign: "right", padding: "16px 0" }}>{pv.b}</td>
								</tr>
							))}
						</tbody>
					</table>
					<p
						style={{
							margin: "0",
							paddingTop: "14px",
							borderTop: "1px solid var(--border)",
							fontSize: "13px",
							lineHeight: "1.5",
							color: "var(--muted-foreground)",
						}}
					>
						{t("light.privNote")}
					</p>
				</div>
				<div>
					<p
						style={{
							margin: "0 0 14px",
							fontSize: "15px",
							fontWeight: "500",
							display: "flex",
							alignItems: "center",
							gap: "8px",
						}}
					>
						<span
							style={{
								width: "8px",
								height: "8px",
								borderRadius: "2px",
								background: "var(--primary)",
							}}
						/>
						{t("light.perfTitle")}
					</p>
					<table style={{ width: "100%", borderCollapse: "collapse", fontSize: "15px" }}>
						<tbody>
							{lightPerf.map((pf) => (
								<tr style={{ borderTop: "1px solid var(--border)" }} key={pf.a}>
									<th
										scope="row"
										style={{
											textAlign: "left",
											fontWeight: "400",
											padding: "16px 0",
											color: "var(--muted-foreground)",
										}}
									>
										{pf.a}
									</th>
									<td
										style={{
											textAlign: "right",
											padding: "16px 0",
											fontSize: "20px",
											fontWeight: "500",
											letterSpacing: "-0.01em",
											fontVariantNumeric: "tabular-nums",
											color: "var(--primary)",
										}}
									>
										{pf.b}
									</td>
								</tr>
							))}
						</tbody>
					</table>
					<p
						style={{
							margin: "0",
							paddingTop: "14px",
							borderTop: "1px solid var(--border)",
							fontSize: "13px",
							lineHeight: "1.5",
							color: "var(--muted-foreground)",
						}}
					>
						{t("light.note")}
					</p>
				</div>
			</div>
		</section>
	);
}
