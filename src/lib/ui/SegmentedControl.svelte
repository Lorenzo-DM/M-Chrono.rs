<script lang="ts">
  import { Timer } from 'lucide-svelte';

  type LucideIcon = typeof Timer;

  type Option = {
    value: string;
    label: string;
    icon?: LucideIcon;
    title?: string;
  };

  let {
    options,
    value,
    onChange,
    ariaLabel,
    class: className = '',
    fullWidth = false,
  } = $props<{
    options: Option[];
    value: string;
    onChange: (value: string) => void;
    ariaLabel: string;
    class?: string;
    fullWidth?: boolean;
  }>();
</script>

<div class={`seg ${fullWidth ? 'w-full' : ''} ${className}`.trim()} role="radiogroup" aria-label={ariaLabel}>
  {#each options as option (option.value)}
    <button
      class={`seg-item ${fullWidth ? 'flex-1' : ''}`}
      data-active={value === option.value}
      role="radio"
      aria-checked={value === option.value}
      aria-label={option.title ?? option.label ?? option.value}
      title={option.title ?? option.label ?? option.value}
      onclick={() => onChange(option.value)}
    >
      {#if option.icon}
        {@const Icon = option.icon}
        <Icon size={14} />
      {:else}
        {option.label}
      {/if}
    </button>
  {/each}
</div>
