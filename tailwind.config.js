/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Background colors - use CSS variables for theme support
        bg: {
          DEFAULT: 'var(--bg)',
          panel: 'var(--bg-panel)',
          card: 'var(--bg-card)',
          hover: 'var(--bg-hover)',
          secondary: 'var(--bg-secondary)',
          active: 'var(--bg-active)',
        },
        // Surface color (alias for card - commonly used for input backgrounds)
        surface: 'var(--bg-card)',
        // Text color alias (for compatibility with text-text pattern)
        text: 'var(--text)',
        // Info color (for informational badges)
        info: {
          DEFAULT: '#58a6ff',
          soft: 'rgba(88, 166, 255, 0.15)',
        },
        // Border colors - use CSS variables
        border: {
          DEFAULT: 'var(--border)',
          muted: 'var(--border-muted)',
        },
        // Text colors - use CSS variables
        txt: {
          DEFAULT: 'var(--text)',
          secondary: 'var(--text-secondary)',
          tertiary: 'var(--text-tertiary)',
          muted: 'var(--text-muted)',
          faint: 'var(--text-faint)',
        },
        // Accent colors - use CSS variables
        accent: {
          DEFAULT: 'var(--accent)',
          soft: 'var(--accent-soft)',
          hover: 'var(--accent-hover)',
        },
        // Status colors - use CSS variables
        success: {
          DEFAULT: 'var(--success)',
          soft: 'var(--success-soft)',
        },
        warning: {
          DEFAULT: 'var(--warning)',
          soft: 'var(--warning-soft)',
        },
        error: {
          DEFAULT: 'var(--error)',
          soft: 'var(--error-soft)',
        },
        // Container type colors (these are consistent across themes)
        type: {
          ad1: '#2f81f7',
          e01: '#3fb950',
          l01: '#d29922',
          raw: '#a371f7',
          ufed: '#38b6ff',
          archive: '#ff7b72',
          tar: '#ffa657',
        },
      },
      // Ring offset color for focus-visible
      ringOffsetColor: {
        bg: 'var(--bg)',
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'sans-serif'],
        mono: ['JetBrains Mono', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'monospace'],
      },
      fontSize: {
        '2xs': ['10px', '14px'],
        xs: ['11px', '16px'],
        sm: ['12px', '18px'],
        base: ['13px', '20px'],
        lg: ['15px', '22px'],
        xl: ['18px', '26px'],
        '2xl': ['22px', '30px'],
      },
      borderRadius: {
        DEFAULT: '6px',
        lg: '10px',
      },
      spacing: {
        '4.5': '18px',
        '11': '44px',
      },
      animation: {
        'pulse-slow': 'pulse 1s ease-in-out infinite',
        'spin-slow': 'spin 1s linear infinite',
        'indeterminate': 'indeterminate 1.5s ease-in-out infinite',
      },
      keyframes: {
        indeterminate: {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(100%)' },
        },
      },
    },
  },
  plugins: [],
}

