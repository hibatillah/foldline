"use client";

import { Dialog } from "@base-ui/react/dialog";
import posthog from "posthog-js";
import { useTranslations } from "next-intl";
import { useState, type CSSProperties, type FormEvent } from "react";

import { ExternalLink } from "./external-link";

const SURVEY_ID = process.env.NEXT_PUBLIC_POSTHOG_SURVEY_ID;
const ISSUES_URL = "https://github.com/hibatillah/foldline/issues";

type Status = "idle" | "sending" | "sent" | "failed";
type SurveyDefinition = { id: string; questionIds: string[] };

const FIELD_STYLE: CSSProperties = {
	width: "100%",
	boxSizing: "border-box",
	padding: "10px 12px",
	borderRadius: 8,
	border: "1px solid color-mix(in oklch, var(--foreground) 14%, transparent)",
	background: "var(--background)",
	color: "var(--foreground)",
	font: "inherit",
	fontSize: 14,
};

/** The text of one form field, or an empty string for a missing one. */
function fieldText(form: FormData, name: string) {
	const value = form.get(name);
	return typeof value === "string" ? value : "";
}

/** Looks the survey up by its id, so the question ids never live in the code. */
function loadSurvey(): Promise<SurveyDefinition | null> {
	return new Promise((resolve) => {
		if (!SURVEY_ID || !posthog.__loaded) return resolve(null);
		posthog.getSurveys((surveys) => {
			const found = surveys.find((survey) => survey.id === SURVEY_ID);
			resolve(
				found
					? { id: found.id, questionIds: found.questions.flatMap((q) => (q.id ? [q.id] : [])) }
					: null,
			);
		}, true);
	});
}

/** A footer button that opens a bug report form; the answer goes to a PostHog survey. */
export function FeedbackDialog({ label }: { label: string }) {
	const t = useTranslations();
	const [status, setStatus] = useState<Status>("idle");

	async function send(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const form = new FormData(event.currentTarget);
		setStatus("sending");
		const survey = await loadSurvey();
		if (!survey || survey.questionIds.length === 0) return setStatus("failed");

		// The survey has two open-text questions: what went wrong, then an optional way to reply.
		const [messageId, contactId] = survey.questionIds;
		posthog.capture("survey sent", {
			$survey_id: survey.id,
			$survey_submission_id: crypto.randomUUID(),
			$survey_completed: true,
			[`$survey_response_${messageId}`]: fieldText(form, "message"),
			...(contactId ? { [`$survey_response_${contactId}`]: fieldText(form, "contact") } : {}),
		});
		setStatus("sent");
	}

	return (
		<Dialog.Root onOpenChange={() => setStatus("idle")}>
			<Dialog.Trigger
				style={{
					padding: 0,
					border: 0,
					background: "none",
					font: "inherit",
					color: "var(--muted-foreground)",
					cursor: "pointer",
					textAlign: "left",
				}}
			>
				{label}
			</Dialog.Trigger>
			<Dialog.Portal>
				<Dialog.Backdrop
					style={{ position: "fixed", inset: 0, zIndex: 50, background: "rgba(15,23,42,0.4)" }}
				/>
				<Dialog.Popup
					style={{
						position: "fixed",
						zIndex: 51,
						top: "50%",
						left: "50%",
						transform: "translate(-50%, -50%)",
						width: "min(440px, calc(100vw - 32px))",
						boxSizing: "border-box",
						padding: 24,
						borderRadius: 16,
						background: "var(--card)",
						color: "var(--foreground)",
						fontFamily: "var(--font-sans)",
						boxShadow: "0 24px 64px -16px rgba(15,23,42,0.45)",
					}}
				>
					<Dialog.Title style={{ margin: 0, fontSize: 18, fontWeight: 600 }}>
						{t("feedback.title")}
					</Dialog.Title>
					<Dialog.Description
						style={{ margin: "8px 0 16px", fontSize: 14, color: "var(--muted-foreground)" }}
					>
						{t("feedback.text")}
					</Dialog.Description>
					{status === "sent" ? (
						<>
							<p style={{ fontSize: 14, margin: "0 0 16px" }}>{t("feedback.thanks")}</p>
							<Dialog.Close style={{ ...FIELD_STYLE, width: "auto", cursor: "pointer" }}>
								{t("feedback.close")}
							</Dialog.Close>
						</>
					) : (
						<form onSubmit={send} style={{ display: "grid", gap: 12 }}>
							<label style={{ display: "grid", gap: 6, fontSize: 13, fontWeight: 500 }}>
								{t("feedback.message")}
								<textarea
									name="message"
									required
									maxLength={2000}
									rows={5}
									style={{ ...FIELD_STYLE, resize: "vertical" }}
								/>
							</label>
							<label style={{ display: "grid", gap: 6, fontSize: 13, fontWeight: 500 }}>
								{t("feedback.contact")}
								<input name="contact" maxLength={200} autoComplete="email" style={FIELD_STYLE} />
							</label>
							{status === "failed" && (
								<p role="alert" style={{ margin: 0, fontSize: 13, color: "var(--destructive)" }}>
									{t("feedback.failed")}
								</p>
							)}
							<div
								style={{
									display: "flex",
									flexWrap: "wrap",
									alignItems: "center",
									justifyContent: "space-between",
									gap: 12,
								}}
							>
								<ExternalLink href={ISSUES_URL} style={{ fontSize: 13, color: "var(--primary)" }}>
									{t("feedback.issue")}
								</ExternalLink>
								<button
									type="submit"
									disabled={status === "sending"}
									style={{
										height: 36,
										padding: "0 16px",
										border: 0,
										borderRadius: 8,
										background: "var(--primary)",
										color: "var(--primary-foreground)",
										fontSize: 14,
										fontWeight: 500,
										cursor: "pointer",
									}}
								>
									{t("feedback.send")}
								</button>
							</div>
						</form>
					)}
				</Dialog.Popup>
			</Dialog.Portal>
		</Dialog.Root>
	);
}
