import { defineStore } from 'pinia';

export interface Theme {
  id: string;
  name: string;
  colors: {
    bgWidget: string;
    bgRow: string;
    textMain: string;
    textPrice: string;
    textQty: string;
    textHeader: string;
    borderColor: string;
    iconBg: string;
  };
  radii: {
    widget: string;
    row: string;
    icon: string;
  };
  fonts: {
    main: string;
  };
}

export const PREDEFINED_THEMES: Record<string, Theme> = {
  default_v2: {
    id: 'default_v2',
    name: 'Torchlight Classic (V2)',
    colors: {
      bgWidget: 'rgba(30, 30, 30, 0.8)',
      bgRow: 'rgba(60, 60, 60, 0.6)',
      textMain: '#ffffff',
      textPrice: '#00ff00',
      textQty: '#aaaaaa',
      textHeader: '#ffd700',
      borderColor: 'rgba(128, 128, 128, 0.5)',
      iconBg: 'rgba(0, 0, 0, 0.3)'
    },
    radii: { widget: '6px', row: '4px', icon: '3px' },
    fonts: { main: "'Segoe UI', Roboto, Helvetica, Arial, sans-serif" }
  },
  dracula: {
    id: 'dracula',
    name: 'Dracula',
    colors: {
      bgWidget: 'rgba(40, 42, 54, 0.9)',
      bgRow: 'rgba(68, 71, 90, 0.7)',
      textMain: '#f8f8f2',
      textPrice: '#50fa7b',
      textQty: '#6272a4',
      textHeader: '#ffb86c',
      borderColor: '#6272a4',
      iconBg: 'rgba(0, 0, 0, 0.5)'
    },
    radii: { widget: '8px', row: '6px', icon: '4px' },
    fonts: { main: "'Fira Code', 'Cascadia Code', monospace" }
  },
  nord: {
    id: 'nord',
    name: 'Nord Frost',
    colors: {
      bgWidget: 'rgba(46, 52, 64, 0.85)',
      bgRow: 'rgba(59, 66, 82, 0.8)',
      textMain: '#eceff4',
      textPrice: '#a3be8c',
      textQty: '#d8dee9',
      textHeader: '#ebcb8b',
      borderColor: '#4c566a',
      iconBg: 'rgba(36, 41, 51, 0.8)'
    },
    radii: { widget: '4px', row: '4px', icon: '2px' },
    fonts: { main: "'Segoe UI', system-ui, sans-serif" }
  }
};

export const useThemeStore = defineStore('theme', {
  state: () => ({
    activeThemeId: 'default_v2',
    customTheme: null as Theme | null,
  }),
  getters: {
    currentTheme: (state): Theme => {
      if (state.activeThemeId === 'custom' && state.customTheme) {
        return state.customTheme;
      }
      return PREDEFINED_THEMES[state.activeThemeId] || PREDEFINED_THEMES['default_v2'];
    }
  },
  actions: {
    setTheme(themeId: string) {
      this.activeThemeId = themeId;
      this.applyTheme();
      this.saveToStorage();
    },
    setCustomTheme(themeJson: string) {
      try {
        const theme: Theme = JSON.parse(themeJson);
        theme.id = 'custom';
        this.customTheme = theme;
        this.activeThemeId = 'custom';
        this.applyTheme();
        this.saveToStorage();
      } catch (e) {
        console.error("Invalid theme JSON", e);
        alert("Invalid theme format!");
      }
    },
    applyTheme() {
      const t = this.currentTheme;
      const root = document.documentElement;

      root.style.setProperty('--theme-bg-widget', t.colors.bgWidget);
      root.style.setProperty('--theme-bg-row', t.colors.bgRow);
      root.style.setProperty('--theme-text-main', t.colors.textMain);
      root.style.setProperty('--theme-text-price', t.colors.textPrice);
      root.style.setProperty('--theme-text-qty', t.colors.textQty);
      root.style.setProperty('--theme-text-header', t.colors.textHeader);
      root.style.setProperty('--theme-border-color', t.colors.borderColor);
      root.style.setProperty('--theme-icon-bg', t.colors.iconBg);

      root.style.setProperty('--theme-radius-widget', t.radii.widget);
      root.style.setProperty('--theme-radius-row', t.radii.row);
      root.style.setProperty('--theme-radius-icon', t.radii.icon);

      root.style.setProperty('--theme-font-main', t.fonts.main);
    },
    saveToStorage() {
      localStorage.setItem('tracker_theme_v2', JSON.stringify({
        activeThemeId: this.activeThemeId,
        customTheme: this.customTheme
      }));
    },
    loadFromStorage() {
      const stored = localStorage.getItem('tracker_theme_v2');
      if (stored) {
        try {
          const parsed = JSON.parse(stored);
          if (parsed.customTheme) {
            this.customTheme = parsed.customTheme;
          }
          if (parsed.activeThemeId) {
            this.activeThemeId = parsed.activeThemeId;
          }
        } catch (e) {}
      }
      this.applyTheme();
    }
  }
});
