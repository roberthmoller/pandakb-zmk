const DEFAULT_WEST_ROOT =
	'/Users/robert/Documents/Codex/2026-11-05/referenced-chatgpt-conversation-this-is-an/work/pandakb-lily';

export type FlashStep = 'setup' | 'build' | 'flash-left' | 'flash-right' | 'done';

export type FirmwareStatus = {
	westRoot: string;
	westRootOk: boolean;
	repoRoot: string | null;
	westBin: string | null;
	dockerOk: boolean;
	bootloaderMounted: boolean;
	leftUf2: string | null;
	rightUf2: string | null;
	busy: boolean;
};

class FirmwareState {
	westRoot = $state(
		(typeof localStorage !== 'undefined' && localStorage.getItem('zmkwiz.westRoot')) ||
			DEFAULT_WEST_ROOT
	);
	step = $state<FlashStep>('setup');
	buildLog = $state('');
	buildStatus = $state<'idle' | 'running' | 'ok' | 'error'>('idle');
	updateFirst = $state(false);
	status = $state<FirmwareStatus | null>(null);
	flashStatus = $state<string | null>(null);
	waitingBootloader = $state(false);

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
}

export const firmwareState = new FirmwareState();
export { DEFAULT_WEST_ROOT };
