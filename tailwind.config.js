import corePreset from '@core-suite/design-tokens/tailwind-preset';

/** @type {import('tailwindcss').Config} */
export default {
  presets: [corePreset],
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
};

