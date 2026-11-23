import { useTranslations } from "next-intl";

import { TwoTone } from "./two-tone";

export type FeatureKey = "open" | "notice" | "plan" | "keep";

/** The numbered heading that opens each feature section. */
export function FeatureHead({ section, number }: { section: FeatureKey; number: string }) {
	const t = useTranslations();

	return (
		<div className="fhead">
			<p className="label" style={{ margin: 0 }}>
				<span style={{ color: "var(--muted-foreground)" }}>{number}</span>
				&nbsp;&nbsp;{t(`f.${section}.label`)}
			</p>
			<div>
				<TwoTone first={t(`f.${section}.a`)} second={t(`f.${section}.b`)} />
				<p className="body" style={{ margin: "36px 0 0" }}>
					{t(`f.${section}.p`)}
				</p>
			</div>
		</div>
	);
}
