/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      // =========================================================================
      // COLORS - Using CSS variables from variables.css
      // =========================================================================
      colors: {
        // Background colors
        bg: {
          DEFAULT: 'var(--color-bg)',
          secondary: 'var(--color-bg-secondary)',
          panel: 'var(--color-bg-panel)',
          card: 'var(--color-bg-card)',
          hover: 'var(--color-bg-hover)',
          active: 'var(--color-bg-active)',
          elevated: 'var(--color-bg-elevated)',
          dark: 'var(--color-bg-dark)',
          subtle: 'var(--color-bg-subtle)',
          muted: 'var(--color-bg-muted)',
          toolbar: 'var(--color-bg-toolbar)',
          header: 'var(--color-bg-header)',
        },
        // Surface colors (layered UI)
        surface: {
          DEFAULT: 'var(--color-surface-primary)',
          secondary: 'var(--color-surface-secondary)',
          tertiary: 'var(--color-surface-tertiary)',
          elevated: 'var(--color-surface-elevated)',
        },
        // Text colors
        txt: {
          DEFAULT: 'var(--color-txt)',
          secondary: 'var(--color-txt-secondary)',
          muted: 'var(--color-txt-muted)',
          faint: 'var(--color-txt-faint)',
          tertiary: 'var(--color-txt-tertiary)',
        },
        // Accent colors (using RGB for opacity support)
        accent: {
          DEFAULT: 'rgb(var(--color-accent-rgb) / <alpha-value>)',
          hover: 'var(--color-accent-hover)',
          soft: 'var(--color-accent-soft)',
        },
        // Status colors
        success: {
          DEFAULT: 'var(--color-success)',
          soft: 'rgba(34, 197, 94, 0.15)',
        },
        warning: {
          DEFAULT: 'var(--color-warning)',
          soft: 'rgba(250, 204, 21, 0.15)',
        },
        error: {
          DEFAULT: 'var(--color-error)',
          soft: 'rgba(239, 68, 68, 0.15)',
        },
        info: {
          DEFAULT: 'var(--color-info)',
          soft: 'rgba(59, 130, 246, 0.15)',
        },
        // Border colors
        border: {
          DEFAULT: 'var(--color-border)',
          subtle: 'var(--color-border-subtle)',
          strong: 'var(--color-border-strong)',
        },
        // Container type colors
        type: {
          ad1: 'var(--color-type-ad1)',
          e01: 'var(--color-type-e01)',
          l01: 'var(--color-type-l01)',
          raw: 'var(--color-type-raw)',
          ufed: 'var(--color-type-ufed)',
          archive: 'var(--color-type-archive)',
        },
      },
      // =========================================================================
      // TYPOGRAPHY
      // =========================================================================
      fontFamily: {
        sans: ['Inter', 'ui-sans-serif', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['JetBrains Mono', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'monospace'],
      },
      fontSize: {
        'tiny': ['var(--font-size-tiny)', { lineHeight: '1.4' }],
        'compact': ['var(--font-size-compact)', { lineHeight: '1.4' }],
        '2xs': ['10px', '14px'],
        xs: ['11px', '16px'],
        sm: ['12px', '18px'],
        base: ['13px', '20px'],
        lg: ['15px', '22px'],
        xl: ['18px', '26px'],
        '2xl': ['22px', '30px'],
      },
      // =========================================================================
      // SPACING - Standard + custom tokens
      // =========================================================================
      spacing: {
        '4.5': '18px',
        '11': '44px',
        'tree-gap': 'var(--tree-item-gap)',
        'tree-icon': 'var(--tree-icon-size)',
      },
      padding: {
        'tree': 'var(--tree-item-padding)',
      },
      gap: {
        'compact': 'var(--gap-compact)',
        'small': 'var(--gap-small)',
        'base': 'var(--gap-base)',
        'tree': 'var(--tree-item-gap)',
      },
      // =========================================================================
      // SIZING
      // =========================================================================
      width: {
        'tree-icon': 'var(--tree-icon-size)',
        'icon-micro': 'var(--icon-size-micro)',
        'icon-compact': 'var(--icon-size-compact)',
        'icon-sm': 'var(--icon-size-small)',
        'icon-base': 'var(--icon-size-base)',
        'icon-lg': 'var(--icon-size-lg)',
      },
      height: {
        'tree-icon': 'var(--tree-icon-size)',
        'icon-micro': 'var(--icon-size-micro)',
        'icon-compact': 'var(--icon-size-compact)',
        'icon-sm': 'var(--icon-size-small)',
        'icon-base': 'var(--icon-size-base)',
        'icon-lg': 'var(--icon-size-lg)',
        'bar-sm': 'var(--bar-height-small)',
        'bar-base': 'var(--bar-height-base)',
        'bar-lg': 'var(--bar-height-lg)',
      },
      minHeight: {
        'bar-sm': 'var(--bar-height-small)',
        'bar-base': 'var(--bar-height-base)',
        'bar-lg': 'var(--bar-height-lg)',
      },
      maxHeight: {
        'settings': 'var(--settings-panel-max-height)',
      },
      maxWidth: {
        'settings': 'var(--settings-panel-width)',
      },
      // =========================================================================
      // BORDER RADIUS
      // =========================================================================
      borderRadius: {
        DEFAULT: 'var(--radius-base)',
        sm: 'var(--radius-sm)',
        md: 'var(--radius-md)',
        lg: 'var(--radius-lg)',
        xl: 'var(--radius-xl)',
        '2xl': 'var(--radius-2xl)',
      },
      // =========================================================================
      // SHADOWS
      // =========================================================================
      boxShadow: {
        'sm': 'var(--shadow-sm)',
        DEFAULT: 'var(--shadow-base)',
        'md': 'var(--shadow-md)',
        'lg': 'var(--shadow-lg)',
        'xl': 'var(--shadow-xl)',
        '2xl': 'var(--shadow-2xl)',
      },
      // =========================================================================
      // Z-INDEX
      // =========================================================================
      zIndex: {
        'dropdown': 'var(--z-dropdown)',
        'sticky': 'var(--z-sticky)',
        'fixed': 'var(--z-fixed)',
        'modal-backdrop': 'var(--z-modal-backdrop)',
        'modal': 'var(--z-modal)',
        'popover': 'var(--z-popover)',
        'tooltip': 'var(--z-tooltip)',
        'notification': 'var(--z-notification)',
      },
      // =========================================================================
      // ANIMATIONS
      // =========================================================================
      animation: {
        'pulse-slow': 'pulse 1s ease-in-out infinite',
        'spin-slow': 'spin 1s linear infinite',
        'indeterminate': 'indeterminate 1.5s ease-in-out infinite',
        'fade-in': 'fadeIn 0.2s ease',
        'slide-up': 'slideUp 0.25s ease',
        'slide-in': 'slideIn 0.3s ease',
      },
      keyframes: {
        indeterminate: {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(100%)' },
        },
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideUp: {
          '0%': { opacity: '0', transform: 'translateY(20px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        slideIn: {
          '0%': { opacity: '0', transform: 'translateX(100%)' },
          '100%': { opacity: '1', transform: 'translateX(0)' },
        },
      },
    },
  },
  plugins: [],
}

