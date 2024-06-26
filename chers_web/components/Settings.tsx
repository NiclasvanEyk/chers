import { useAdjustableSettings } from "@/lib/settings";
import { ReactNode, useEffect, useId, useRef } from "react";

export interface TriggerProps {
	color: "White" | "Black";
	className?: string;
	onClick: () => unknown;
}

export function SettingsTrigger(props: TriggerProps) {
	const { className, color, ...rest } = props;

	let colorClasses =
		color === "Black"
			? "bg-chess-brown/60 hover:bg-chess-brown/50 dark:border-chess-beige/50 border-yellow-900"
			: "bg-chess-beige/60 hover:bg-chess-beige/50 dark:border-chess-beige border-chess-brown";

	return (
		<button
			className={[
				"h-4 w-4 transition-colors hover:bg-transparent border-2 rounded-full border-dashed shadow-md",
				className ?? "",
				colorClasses,
			].join(" ")}
			title="Open Settings"
			{...rest}
		/>
	);
}

export function Settings(props: { open: boolean; onClose: () => void }) {
	const settings = useAdjustableSettings();
	const dialog = useRef<HTMLDialogElement | null>(null);

	useEffect(() => {
		if (props.open) {
			dialog.current?.showModal();
		} else {
			dialog.current?.close();
		}
	}, [props.open]);

	return (
		<dialog
			ref={dialog}
			onClose={props.onClose}
			className="fixed backdrop-blur-xl bg-white/70 text-black dark:bg-black/80 dark:text-white rounded shadow p-9 inset-0 z-20 backdrop:bg-transparent"
		>
			<button
				onClick={props.onClose}
				className="text-2xl absolute top-0 right-0 h-12 w-12"
				aria-label="Close Settings"
			>
				&times;
			</button>

			<h2 className="text-2xl mb-7">Settings</h2>

			<div className="space-y-7">
				<Section heading="Gameplay">
					<Toggle
						value={settings.all.highlightLegalMoves}
						onChange={(value) => settings.adjust("highlightLegalMoves", value)}
					>
						Highlight possible moves using dark circles.
					</Toggle>
					<Toggle
						value={settings.all.displayLabels}
						onChange={(value) => settings.adjust("displayLabels", value)}
					>
						Display rank and file labels.
					</Toggle>
					<Toggle
						functional={false}
						value={settings.all.displayCapturedPieces}
						onChange={(value) =>
							settings.adjust("displayCapturedPieces", value)
						}
					>
						Display captured pieces.
					</Toggle>
				</Section>

				<Section heading="Keyboard Navigation">
					<Toggle
						functional={false}
						value={settings.all.onlyLegalTabTargets}
						onChange={(value) => settings.adjust("onlyLegalTabTargets", value)}
					>
						Skip non-selectable cells when tabbing.
					</Toggle>
				</Section>
			</div>
		</dialog>
	);
}

function Section(props: { heading: string; children: ReactNode }) {
	return (
		<section>
			<h3 className="text-xl mb-1">{props.heading}</h3>

			{props.children}
		</section>
	);
}

function Toggle(props: {
	children: ReactNode;
	functional?: boolean;
	value: boolean;
	onChange: (value: boolean) => void;
}) {
	const placeholder = props.functional === false;
	const id = useId();

	return (
		<label
			htmlFor={id}
			className={`flex flex-row items-center select-none ${
				placeholder ? "opacity-50" : ""
			}`}
		>
			<input
				type="checkbox"
				id={id}
				className="mr-3"
				disabled={placeholder}
				checked={props.value}
				onChange={(event) => props.onChange(event.target.checked)}
			/>
			{props.children} {placeholder ? "-- todo" : null}
		</label>
	);
}
