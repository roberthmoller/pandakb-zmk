const DEFAULT_WEST_ROOT = '~/.zmk';

function initialWestRoot(): string {
	if (typeof localStorage === 'undefined') return DEFAULT_WEST_ROOT;
	const stored = localStorage.getItem('zmkwiz.westRoot');
	if (!stored) return DEFAULT_WEST_ROOT;
	// Migrate away from the old Codex pandakb-lily path.
	if (stored.includes('referenced-chatgpt') || stored.includes('pandakb-lily')) {
		localStorage.setItem('zmkwiz.westRoot', DEFAULT_WEST_ROOT);
		return DEFAULT_WEST_ROOT;
	}
	return stored;
}

/** Happy path phases stay quiet; wizard* means the user must act. */
export type FlashPhase =
	| 'idle'
	| 'building'
	| 'flash-left'
	| 'flash-right'
	| 'wizard-right'
	| 'wizard-bootloader'
	| 'done'
	| 'error';

export type WizardKind = 'right-cable' | 'bootloader' | null;

export type FirmwareStatus = {
	westRoot: string;
	westRootOk: boolean;
	repoRoot: string | null;
	westBin: string | null;
	dockerOk: boolean;
	bootloaderMounted: boolean;
	bootloaderPath: string | null;
	leftUf2: string | null;
	rightUf2: string | null;
	sourcesStale: boolean;
	busy: boolean;
};

class FirmwareState {
	westRoot = $state(initialWestRoot());
	phase = $state<FlashPhase>('idle');
	wizard = $state<WizardKind>(null);
	wizardHalf = $state<'left' | 'right'>('left');
	buildLog = $state('');
	updateFirst = $state(false);
	forceBuild = $state(false);
	showAdvanced = $state(false);
	showLog = $state(false);
	status = $state<FirmwareStatus | null>(null);
	message = $state<string | null>(null);
	running = $state(false);

	persistWestRoot() {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('zmkwiz.westRoot', this.westRoot);
		}
	}

	appendLog(line: string) {
		this.buildLog = this.buildLog ? `${this.buildLog}\n${line}` : line;
	}

	resetLog() {
		this.buildLog = '';
	}

	clearWizard() {
		this.wizard = null;
	}
}

export const firmwareState = new FirmwareState();
export { DEFAULT_WEST_ROOT };
