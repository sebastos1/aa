<script>
	let { src, alt = "", text = "" } = $props();
</script>

<div class="icon-text-container">
	<img {src} {alt} class="icon" />
	<span>{text}</span>
</div>

<style>
	.icon-text-container {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		font-size: inherit;
		font-family: inherit;
		vertical-align: middle;
	}
	.icon {
		width: 1.25em;
		height: 1.25em;
		image-rendering: pixelated;
	}
</style>