import {
	call_rpc,
	create_rpc_connection,
	type RpcConnection
} from '@zmkfirmware/zmk-studio-ts-client';
import { LockState } from '@zmkfirmware/zmk-studio-ts-client/core';
import type { RpcTransport } from '@zmkfirmware/zmk-studio-ts-client/transport/index';

export type StudioStatus = 'idle' | 'connecting' | 'connected' | 'error';

class StudioConnection {
	status = $state<StudioStatus>('idle');
	deviceName = $state<string | null>(null);
	lockState = $state<LockState>(LockState.ZMK_STUDIO_CORE_LOCK_STATE_LOCKED);
	error = $state<string | null>(null);
	conn = $state<RpcConnection | null>(null);

	#abort: AbortController | null = null;
	#transport: RpcTransport | null = null;

	get unlocked(): boolean {
		return this.lockState === LockState.ZMK_STUDIO_CORE_LOCK_STATE_UNLOCKED;
	}

	async connect() {
		if (typeof navigator === 'undefined' || !('serial' in navigator)) {
			this.status = 'error';
			this.error = 'Web Serial requires Chrome or Edge on desktop.';
			return;
		}

		this.disconnect();
		this.status = 'connecting';
		this.error = null;
		this.#abort = new AbortController();

		try {
			const { connect: serialConnect } = await import(
				'@zmkfirmware/zmk-studio-ts-client/transport/serial'
			);
			const transport = await serialConnect();
			this.#transport = transport;
			const conn = create_rpc_connection(transport, { signal: this.#abort.signal });

			const info = await Promise.race([
				call_rpc(conn, { core: { getDeviceInfo: true } }).then((r) => r.core?.getDeviceInfo),
				new Promise<undefined>((resolve) => setTimeout(() => resolve(undefined), 1500))
			]);

			if (!info) {
				throw new Error('No Studio response — plug in the central half over USB and retry.');
			}

			this.conn = conn;
			this.deviceName = info.name || 'ZMK device';
			this.status = 'connected';

			const lock = await call_rpc(conn, { core: { getLockState: true } });
			if (lock.core?.getLockState !== undefined) {
				this.lockState = lock.core.getLockState;
			}

			this.#listenNotifications(conn);
		} catch (err) {
			this.status = 'error';
			this.error = err instanceof Error ? err.message : String(err);
			this.disconnect();
		}
	}

	disconnect() {
		this.#abort?.abort();
		this.#abort = null;
		try {
			this.#transport?.abortController.abort();
		} catch {
			/* already closed */
		}
		this.#transport = null;
		this.conn = null;
		this.deviceName = null;
		this.lockState = LockState.ZMK_STUDIO_CORE_LOCK_STATE_LOCKED;
		if (this.status !== 'error') this.status = 'idle';
	}

	async rpc(req: Parameters<typeof call_rpc>[1]) {
		if (!this.conn) throw new Error('Not connected');
		return call_rpc(this.conn, req);
	}

	async #listenNotifications(conn: RpcConnection) {
		const reader = conn.notification_readable.getReader();
		try {
			while (true) {
				const { done, value } = await reader.read();
				if (done) break;
				const lock = value?.core?.lockStateChanged;
				if (lock !== undefined) this.lockState = lock;
			}
		} catch {
			/* aborted */
		} finally {
			reader.releaseLock();
			if (this.conn === conn) {
				this.status = 'idle';
				this.conn = null;
				this.deviceName = null;
			}
		}
	}
}

export const studioConnection = new StudioConnection();
