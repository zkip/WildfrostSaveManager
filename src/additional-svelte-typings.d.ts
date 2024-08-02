// https://svelte.dev/docs/typescript#enhancing-built-in-dom-types
declare namespace svelteHTML {
	// enhance attributes
	interface HTMLAttributes<T> {
		thumb?: boolean;
	}
}