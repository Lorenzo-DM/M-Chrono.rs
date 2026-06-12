<script lang="ts">
  type Option = {
    value: string;
    label: string;
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
      aria-label={option.label ?? option.value}
      title={option.title ?? option.label ?? option.value}
    onclick={() => onChange(option.value)}
    >
      {option.label}
    </button>
  {/each}
</div>
