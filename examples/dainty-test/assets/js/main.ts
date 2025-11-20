import Alpine from "alpinejs";
import htmx from "htmx.org";
import "@phosphor-icons/webcomponents/PhCircleHalf";
import "@phosphor-icons/webcomponents/PhSun";
import "@phosphor-icons/webcomponents/PhMoon";
import "@phosphor-icons/webcomponents/PhDotsThreeOutlineVertical";

import { createIcons, Menu, PanelLeftClose, PanelLeftOpen, PanelRightClose, PanelRightOpen } from "lucide";

type SidebarSide = "left" | "right";
type LayoutTheme = "system" | "light" | "dark";
type DeviceType = "mobile" | "desktop";

interface LayoutStateContext {
  leftSidebar: boolean;
  rightSidebar: boolean;
  leftWidth: number;
  rightWidth: number;
  resizing: SidebarSide | null;
  startX: number;
  startWidth: number;
  theme: LayoutTheme;
  contextPath: string;
  deviceType: DeviceType;
  userId: string;
  saveTimeout: ReturnType<typeof globalThis.setTimeout> | null;
  init(): Promise<void>;
  loadState(): Promise<void>;
  saveState(updates: Record<string, unknown>): void;
  startResize(side: SidebarSide, e: MouseEvent): void;
  doResize(e: MouseEvent): void;
  stopResize(): void;
  cycleTheme(): void;
  applyTheme(): void;
  $watch: <T>(property: string, callback: (value: T) => void) => void;
}

createIcons({
  icons: {
    PanelLeftClose,
    PanelLeftOpen,
    PanelRightClose,
    PanelRightOpen,
    Menu,
  },
});

// Define layout state component
Alpine.data("layoutState", () => ({
  leftSidebar: window.innerWidth >= 1024,
  rightSidebar: window.innerWidth >= 1024,
  leftWidth: 320,
  rightWidth: 320,
  resizing: null as SidebarSide | null,
  startX: 0,
  startWidth: 0,
  theme: "system" as LayoutTheme,
  contextPath: window.location.pathname,
  deviceType: (window.innerWidth < 1024 ? "mobile" : "desktop") as DeviceType,
  userId: "default",
  saveTimeout: null as ReturnType<typeof globalThis.setTimeout> | null,

  async init(this: LayoutStateContext) {
    await this.loadState();

    // Set up watchers after state is loaded
    this.$watch("leftSidebar", (value: boolean) => {
      this.saveState({ left_sidebar_open: value });
    });
    this.$watch("rightSidebar", (value: boolean) => {
      this.saveState({ right_sidebar_open: value });
    });
    this.$watch("theme", (value: LayoutTheme) => {
      this.saveState({ theme: value });
    });
  },

  async loadState(this: LayoutStateContext) {
    try {
      const response = await fetch(
        `/api/layout?path=${encodeURIComponent(this.contextPath)}&device=${this.deviceType}`,
        {
          headers: { "X-API-Key": this.userId },
        },
      );
      if (response.ok) {
        const settings = await response.json();

        // Apply loaded settings
        if (window.innerWidth >= 1024) {
          this.leftSidebar = settings.left_sidebar_open ?? true;
          this.rightSidebar = settings.right_sidebar_open ?? true;
        }
        this.leftWidth = settings.left_width ?? 320;
        this.rightWidth = settings.right_width ?? 320;
        this.theme = settings.theme ?? "system";
        this.applyTheme();
      }
    } catch (error) {
      console.error("Failed to load layout state:", error);
    }
  },

  saveState(this: LayoutStateContext, updates: Record<string, unknown>) {
    // Debounce saves to avoid hammering the server
    if (this.saveTimeout) {
      globalThis.clearTimeout(this.saveTimeout);
    }

    this.saveTimeout = globalThis.setTimeout(() => {
      fetch("/api/layout", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-API-Key": this.userId,
        },
        body: JSON.stringify({
          path: this.contextPath,
          device: this.deviceType,
          ...updates,
        }),
      }).catch((error) => {
        console.error("Failed to save layout state:", error);
      });
    }, 300);
  },

  startResize(this: LayoutStateContext, side: SidebarSide, e: MouseEvent) {
    this.resizing = side;
    this.startX = e.clientX;
    this.startWidth = side === "left" ? this.leftWidth : this.rightWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  },

  doResize(this: LayoutStateContext, e: MouseEvent) {
    if (!this.resizing) return;
    const delta = this.resizing === "left" ? e.clientX - this.startX : this.startX - e.clientX;
    const newWidth = this.startWidth + delta;
    if (newWidth <= 5) {
      if (this.resizing === "left") {
        this.leftSidebar = false;
        this.leftWidth = 320;
        this.saveState({ left_sidebar_open: false });
      } else {
        this.rightSidebar = false;
        this.rightWidth = 320;
        this.saveState({ right_sidebar_open: false });
      }
      this.stopResize();
      return;
    }
    if (this.resizing === "left") {
      this.leftWidth = newWidth;
    } else {
      this.rightWidth = newWidth;
    }
  },

  stopResize(this: LayoutStateContext) {
    if (this.resizing) {
      // Save width on resize complete
      if (this.resizing === "left") {
        this.saveState({ left_width: this.leftWidth });
      } else {
        this.saveState({ right_width: this.rightWidth });
      }
    }
    this.resizing = null;
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  },

  cycleTheme(this: LayoutStateContext) {
    if (this.theme === "system") {
      this.theme = "light";
    } else if (this.theme === "light") {
      this.theme = "dark";
    } else {
      this.theme = "system";
    }
    this.applyTheme();
    this.saveState({ theme: this.theme });
  },

  applyTheme(this: LayoutStateContext) {
    if (this.theme === "system") {
      const systemTheme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
      document.documentElement.setAttribute("data-theme", systemTheme);
    } else {
      document.documentElement.setAttribute("data-theme", this.theme);
    }
  },
}));

window.Alpine = Alpine;
window.htmx = htmx;

Alpine.start();

console.log("Alpine.js and HTMX initialized");
