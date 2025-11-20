import type Alpine from "alpinejs";
import type htmx from "htmx.org";

declare global {
  interface Window {
    Alpine: typeof Alpine;
    htmx: typeof htmx;
  }
}
