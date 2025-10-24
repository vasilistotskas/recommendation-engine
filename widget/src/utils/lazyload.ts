/**
 * Lazy load utility for images using IntersectionObserver
 */

export class LazyLoader {
  private observer: IntersectionObserver | null = null;
  private images: Set<HTMLImageElement> = new Set();

  constructor() {
    if ("IntersectionObserver" in window) {
      this.observer = new IntersectionObserver(
        (entries) => {
          entries.forEach((entry) => {
            if (entry.isIntersecting) {
              const img = entry.target as HTMLImageElement;
              this.loadImage(img);
              this.observer?.unobserve(img);
              this.images.delete(img);
            }
          });
        },
        {
          rootMargin: "50px", // Start loading 50px before image enters viewport
          threshold: 0.01,
        },
      );
    }
  }

  /**
   * Observe images for lazy loading
   */
  public observe(container: HTMLElement): void {
    if (!this.observer) {
      // Fallback: load all images immediately if IntersectionObserver not supported
      container
        .querySelectorAll<HTMLImageElement>("img[data-src]")
        .forEach((img) => {
          this.loadImage(img);
        });
      return;
    }

    container
      .querySelectorAll<HTMLImageElement>("img[data-src]")
      .forEach((img) => {
        this.images.add(img);
        this.observer!.observe(img);
      });
  }

  /**
   * Load a single image
   */
  private loadImage(img: HTMLImageElement): void {
    const src = img.dataset.src;
    if (!src) return;

    img.src = src;
    img.removeAttribute("data-src");

    // Add loaded class for fade-in animation
    img.addEventListener("load", () => {
      img.classList.add("gs-image-loaded");
    });

    // Handle error
    img.addEventListener("error", () => {
      img.classList.add("gs-image-error");
      // Set placeholder image
      img.src = "/placeholder.png";
    });
  }

  /**
   * Unobserve all images and disconnect
   */
  public disconnect(): void {
    if (this.observer) {
      this.images.forEach((img) => {
        this.observer!.unobserve(img);
      });
      this.observer.disconnect();
      this.images.clear();
    }
  }
}

/**
 * Prefetch link on hover
 */
export function setupPrefetchOnHover(container: HTMLElement): void {
  const prefetchedLinks = new Set<string>();

  container.querySelectorAll<HTMLAnchorElement>("a[href]").forEach((link) => {
    link.addEventListener(
      "mouseenter",
      () => {
        const href = link.href;

        // Only prefetch if not already prefetched
        if (prefetchedLinks.has(href)) return;

        // Create prefetch link
        const prefetch = document.createElement("link");
        prefetch.rel = "prefetch";
        prefetch.href = href;
        prefetch.as = "document";

        document.head.appendChild(prefetch);
        prefetchedLinks.add(href);
      },
      { once: true },
    );
  });
}
