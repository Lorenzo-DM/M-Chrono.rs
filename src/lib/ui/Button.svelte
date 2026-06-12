<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'start' | 'tap' | 'finish' | 'ghost' | 'danger' | 'default';
  type Size = 'sm' | 'md' | 'lg';

  let {
    variant = 'default',
    size = 'md',
    type = 'button',
    disabled = false,
    title,
    style,
    ariaLabel,
    ariaExpanded,
    ariaPressed,
    class: className = '',
    onclick,
    children,
  } = $props<{
    variant?: Variant;
    size?: Size;
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    title?: string;
    style?: string;
    ariaLabel?: string;
    ariaExpanded?: boolean | 'true' | 'false';
    ariaPressed?: boolean | 'true' | 'false';
    class?: string;
    onclick?: (event: MouseEvent) => void;
    children?: Snippet;
  }>();

  const variantClass: Record<Variant, string> = {
    default: '',
    primary: 'btn-primary',
    start: 'btn-accent-start',
    tap: 'btn-accent-tap',
    finish: 'btn-accent-finish',
    ghost: 'btn-ghost',
    danger: 'btn-danger',
  };

  const sizeClass: Record<Size, string> = {
    sm: 'btn-sm',
    md: 'btn-md',
    lg: 'btn-lg',
  };

  let normalizedVariant = $derived.by<Variant>(() => variant as Variant);
  let normalizedSize = $derived.by<Size>(() => size as Size);
</script>

<button
  class={`btn-base ${variantClass[normalizedVariant]} ${sizeClass[normalizedSize]} ${className}`.trim()}
  type={type}
  disabled={disabled}
  title={title}
  style={style}
  aria-label={ariaLabel}
  aria-expanded={ariaExpanded}
  aria-pressed={ariaPressed}
  onclick={onclick}
>
  {@render children?.()}
</button>
