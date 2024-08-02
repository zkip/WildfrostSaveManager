<script lang="ts">
	import { register } from '@tauri-apps/api/globalShortcut';
	import { appWindow, LogicalSize } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';
	import type { Snapshot } from '../types/snapshot';
	import {
		clear,
		get_current,
		get_profile,
		get_profiles,
		get_snapshots,
		restore,
		set_profile,
		snapshot,
	} from '../lib/api';
	import dayjs from 'dayjs';
	import { browser } from '$app/environment';
	import Icomoon from '../components/Icomoon.svelte';
	import { exit } from '@tauri-apps/api/process';

	let saveAudio = browser ? new Audio('sound/save.wav') : undefined;
	let loadAudio = browser ? new Audio('sound/load.wav') : undefined;

	async function registerShortcuts() {
		register('Alt+Q', () => {
			save.call(snapshots[0]);
		});

		register('Alt+F', () => {
			load.call(snapshots[0]);
		});
	}

	onMount(() => {
		if (browser) {
			registerShortcuts();
		}
		addEventListener('mousedown', async (e) => {
			if (
				e
					.composedPath()
					.some(
						(target) =>
							target instanceof HTMLElement &&
							target.hasAttribute('thumb')
					)
			) {
				await appWindow.startDragging();
			}
		});
		addEventListener('contextmenu', async (e) => {
			e.preventDefault();
		});

		const boundElement = document.querySelector('.container')!;
		const observer = new ResizeObserver(() => {
			const { width, height } = boundElement.getBoundingClientRect();
			appWindow.setSize(new LogicalSize(width, height));
		});
		observer.observe(boundElement);
	});

	let snapshots: Snapshot[] = [];
	let profiles: string[] = [];
	let current: number | undefined;
	let profile = '';

	async function init() {
		current = await get_current();
		profile = await get_profile();
		profiles = await get_profiles();

		await syncSnapshots();
	}

	async function syncSnapshots() {
		const init_snapshots: Snapshot[] = new Array(4).fill(0).map((_, i) => ({
			name: i === 0 ? `快速存档` : `存档${i}`,
			activate: false,
			index: i,
		}));
		let shots = await get_snapshots();

		const arr: (Snapshot | undefined)[] = [];
		shots.map((snapshot) => (arr[snapshot.index] = snapshot));
		snapshots = init_snapshots.map((snapshot, index) => ({
			index: arr[index]?.index ?? snapshot.index,
			name: arr[index]?.name ?? snapshot.name,
			date: arr[index]?.date,
			activate: Boolean(arr[index]),
		}));
	}

	init();

	async function save(this: Snapshot) {
		const snap = {
			...this,
			activate: true,
			date: new Date().toString(),
		} satisfies Snapshot;
		await snapshot(snap);
		Object.assign(this, snap);
		snapshots = snapshots.slice();

		saveAudio!.volume = 0.7;
		saveAudio!.currentTime = 0;
		saveAudio!.play();
	}
	async function load(this: Snapshot) {
		if (!this.activate) return;
		await restore(this);
		snapshots = snapshots.slice();
		current = this.index;

		loadAudio!.volume = 0.5;
		loadAudio!.currentTime = 0;
		loadAudio!.play();
	}
	async function clearSave(this: Snapshot) {
		if (!this.activate) return;
		await clear(this.name);
		this.activate = false;
		this.date = undefined;
		snapshots = snapshots.slice();
	}

	async function onSnapshotAction(this: Snapshot, event: MouseEvent) {
		if (event.button === 0) save.call(this);
		if (event.button === 2) load.call(this);
		if (event.button === 1) clearSave.call(this);
	}

	async function onProfileChange(event: {
		currentTarget: EventTarget & HTMLSelectElement;
	}) {
		profile = event.currentTarget.value;

		await set_profile(profile);
		await syncSnapshots();
		current = await get_current();
	}
</script>

<div class="container">
	<div class="snapshot">
		{#each snapshots as snapshot}
			{@const { activate, name } = snapshot}
			<button
				class:activate
				on:pointerdown={onSnapshotAction.bind(snapshot)}
			>
				{#if current === snapshot.index}
					<span>+</span>
				{/if}
				<span>{name}</span>{#if snapshot.date}<span class="time"
						>{dayjs(snapshot.date).format('HH:mm:ss')}</span
					>{/if}
				<br />
				{#if snapshot.date}<span class="date"
						>{dayjs(snapshot.date).format('YYYY-MM-DD')}</span
					>{/if}
			</button>
		{/each}
		<select
			name="profile"
			id="profile"
			class="profile"
			value={profile}
			on:change={onProfileChange}
		>
			{#each profiles as profile}
				<option value={profile}>{profile}</option>
			{/each}
		</select>
	</div>
	<div class="thumb" thumb>
		<Icomoon name="drag_indicator" title="quit" />
	</div>
	<div class="tips">
		<i>快速存档：<b>Alt + Q</b></i>
		&nbsp;
		<i>快速读档：<b>Alt + F</b></i>
		&nbsp;
		<i>鼠标左击档位：存档；</i>
		<i>鼠标右击档位：读档；</i>
		<i>鼠标中击档位：清除；</i>
		<button class="quit" on:click={() => exit(0)}>退出</button>
	</div>
</div>

<svelte:head>
	<title>雪居之地存档管理</title>
</svelte:head>

<style>
	.quit {
		background-color: transparent;
		border: none;
		float: right;
		cursor: pointer;
		font-size: 1em;
		color: inherit;
	}
	.quit:hover {
		color: rgba(255, 68, 0, 0.5);
	}
	.thumb > :global(svg) {
		width: 18px;
	}
	.thumb {
		display: flex;
		-webkit-app-region: drag;
		cursor: move;
		justify-content: center;
	}

	:root {
		--outer-radius: 6px;
	}
	.date {
		font-size: 8px;
		opacity: 0.5;
	}
	.time {
		padding: 4px 2px;
		padding-left: 8px;
		opacity: 0.5;
	}
	.snapshot > .activate:hover .time {
		font-size: 12px;
	}
	.snapshot > .activate:hover .date {
		font-size: 14px;
		opacity: 1;
	}

	.tips {
		flex-basis: 100%;
		background-color: transparent;
		font-size: 12px;
		color: #888;
		padding: 8px;
		grid-column: 1 / span 2;
		background-color: rgba(236, 236, 236, 0.8);
		-webkit-app-region: drag;
		border-radius: 0 0 var(--outer-radius) var(--outer-radius);
	}
	.container {
		display: grid;
		gap: 0;
		box-sizing: border-box;
		border: 1px solid rebeccapurple;
		border-radius: var(--outer-radius);
		grid-template-columns: 1fr 32px;
		grid-template-rows: 1fr auto;
		background-color: rgba(0, 0, 0, 0.2);
	}
	.snapshot {
		flex: 1;
		/* background-color: rebeccapurple; */
		/* max-height: 120px;
		max-width: 120px; */
		-webkit-app-region: drag;
		padding: 6px;
		display: grid;
		grid-template-columns: repeat(4, 1fr) min-content;
		grid-auto-flow: row;
		gap: 6px;
		border-radius: var(--outer-radius) 0 0 0;
	}

	.snapshot > * {
		padding: 6px 24px;
		border-radius: 4px;
		border: none;
		-webkit-app-region: no-drag;
		white-space: nowrap;
	}
	.snapshot > .activate:hover {
		background-color: rgb(228, 228, 228);
		/* opacity: 0.8; */
		/* transform: rotate(-2deg); */
	}
	.snapshot > .activate:active {
		opacity: 1;
		/* transform: scale(0.9) rotate(3deg); */
		transform: scale(0.9);
	}
	.snapshot > :not(.activate) {
		cursor: pointer;
		color: rgb(54, 54, 54);
		background-color: rgb(146, 146, 146, 0.8);
	}
	.profile {
		padding: 6px 12px;
	}
</style>
