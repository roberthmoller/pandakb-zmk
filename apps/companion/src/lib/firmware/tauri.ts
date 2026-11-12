import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { FirmwareStatus } from '#lib/state/firmware.svelte';

export type FlashResult = {
	half: string;
	from: string;
	to: string;
};

export type KeymapFile = {
	path: string;
	repoRoot: string;
	contents: string;
};

export function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function getFirmwareStatus(westRoot: string): Promise<FirmwareStatus> {
	return invoke<FirmwareStatus>('firmware_status', { westRoot });
}

export async function westUpdate(westRoot: string): Promise<void> {
	return invoke('firmware_west_update', { westRoot });
}

export async function buildFirmware(westRoot: string, updateFirst: boolean): Promise<void> {
	return invoke('firmware_build', { westRoot, updateFirst });
}

export async function flashHalf(half: 'left' | 'right', westRoot: string): Promise<FlashResult> {
	return invoke<FlashResult>('firmware_flash', { half, westRoot });
}

export async function waitBootloader(timeoutSecs = 90): Promise<boolean> {
	return invoke<boolean>('firmware_wait_bootloader', { timeoutSecs });
}

export async function onFirmwareLog(handler: (line: string) => void): Promise<UnlistenFn> {
	return listen<string>('firmware://log', (event) => handler(event.payload));
}

export async function onFirmwareDone(handler: (kind: string) => void): Promise<UnlistenFn> {
	return listen<string>('firmware://done', (event) => handler(event.payload));
}

/** Read `config/Lily58.keymap` from the detected pandakb-zmk repo. */
export async function getFirmwareKeymap(): Promise<KeymapFile> {
	return invoke<KeymapFile>('firmware_keymap_get');
}

/** Persist companion edits back to `config/Lily58.keymap`. */
export async function setFirmwareKeymap(contents: string): Promise<KeymapFile> {
	return invoke<KeymapFile>('firmware_keymap_set', { contents });
}
