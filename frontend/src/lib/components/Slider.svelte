<script>
	import { onMount, onDestroy } from 'svelte';

	let slides = $state([]);
	let current = $state(0);
	let interval = $state(null);

	onMount(async () => {
		try {
			const res = await fetch('/api/sliders');
			if (res.ok) slides = await res.json();
		} catch { /* empty */ }
		startAutoplay();
	});

	onDestroy(() => {
		if (interval) clearInterval(interval);
	});

	function startAutoplay() {
		if (interval) clearInterval(interval);
		interval = setInterval(() => {
			if (slides.length > 1) {
				current = (current + 1) % slides.length;
			}
		}, 5000);
	}

	function goTo(index) {
		current = index;
		startAutoplay();
	}

	function prev() {
		current = (current - 1 + slides.length) % slides.length;
		startAutoplay();
	}

	function next() {
		current = (current + 1) % slides.length;
		startAutoplay();
	}
</script>

{#if slides.length > 0}
	<section class="slider" role="region" aria-label="Slider">
		<div class="slider-viewport">
			{#each slides as slide, i}
				<div class="slide" class:active={i === current}>
					{#if slide.link_url}
						<a href={slide.link_url} class="slide-link">
							<img src={slide.image_url} alt={slide.title} class="slide-img" />
							<div class="slide-content">
								<h2 class="slide-title">{slide.title}</h2>
								{#if slide.description}
									<p class="slide-desc">{slide.description}</p>
								{/if}
							</div>
						</a>
					{:else}
						<img src={slide.image_url} alt={slide.title} class="slide-img" />
						<div class="slide-content">
							<h2 class="slide-title">{slide.title}</h2>
							{#if slide.description}
								<p class="slide-desc">{slide.description}</p>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		</div>

		{#if slides.length > 1}
			<button class="slider-arrow slider-prev" onclick={prev} aria-label="Önceki">&#x2039;</button>
			<button class="slider-arrow slider-next" onclick={next} aria-label="Sonraki">&#x203A;</button>

			<div class="slider-dots">
				{#each slides as _, i}
					<button
						class="dot"
						class:active={i === current}
						onclick={() => goTo(i)}
						aria-label="Slide {i + 1}"
					></button>
				{/each}
			</div>
		{/if}
	</section>
{/if}

<style>
	.slider {
		position: relative;
		overflow: hidden;
		border-radius: var(--radius-lg);
		margin-bottom: 32px;
		background: var(--color-bg-secondary);
	}
	.slider-viewport {
		position: relative;
		width: 100%;
		aspect-ratio: 21 / 9;
	}
	.slide {
		position: absolute;
		inset: 0;
		opacity: 0;
		transition: opacity 0.6s ease;
	}
	.slide.active { opacity: 1; z-index: 1; }
	.slide-link {
		display: block;
		width: 100%;
		height: 100%;
		color: inherit;
	}
	.slide-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.slide-content {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		padding: 32px;
		background: linear-gradient(transparent, rgba(0,0,0,0.7));
		color: #fff;
	}
	.slide-title {
		font-size: 1.75rem;
		font-weight: 700;
		margin-bottom: 4px;
		text-shadow: 0 2px 4px rgba(0,0,0,0.3);
	}
	.slide-desc {
		font-size: 1rem;
		opacity: 0.9;
		text-shadow: 0 1px 3px rgba(0,0,0,0.3);
	}
	.slider-arrow {
		position: absolute;
		top: 50%;
		transform: translateY(-50%);
		z-index: 2;
		width: 48px;
		height: 48px;
		border: none;
		border-radius: 50%;
		background: rgba(0,0,0,0.4);
		color: #fff;
		font-size: 2rem;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background var(--transition-fast);
		line-height: 1;
	}
	.slider-arrow:hover { background: rgba(0,0,0,0.7); }
	.slider-prev { left: 12px; }
	.slider-next { right: 12px; }
	.slider-dots {
		position: absolute;
		bottom: 12px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 2;
		display: flex;
		gap: 8px;
	}
	.dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		border: 2px solid #fff;
		background: transparent;
		cursor: pointer;
		transition: background var(--transition-fast);
		padding: 0;
	}
	.dot.active { background: #fff; }

	@media (max-width: 640px) {
		.slider-viewport { aspect-ratio: 16 / 9; }
		.slide-content { padding: 16px; }
		.slide-title { font-size: 1.125rem; }
		.slide-desc { font-size: 0.875rem; }
		.slider-arrow { width: 36px; height: 36px; font-size: 1.5rem; }
	}
</style>
