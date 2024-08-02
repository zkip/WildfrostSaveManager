<script lang="ts">
	export let title;
	export let disableFill = false;
	import iconSet from './selection.json';
	export let name: string;
	
	const currentIcon = iconSet.icons.find(
		(item) => item.properties.name === name
	);


	const paths =
		currentIcon?.icon.paths.map((path, index) => {
			const pathProps = {
				d: path,
				...(!disableFill ? currentIcon?.icon.attrs[index] : {}),
			};
			return pathProps;
		}) || [];
</script>

<svg viewBox="0 0 1024 1024" {...$$props}>
	{#each paths as path}
		<path d={path.d} />
	{/each}
	{#if title}
		<title>{title}</title>
	{/if}
</svg>
