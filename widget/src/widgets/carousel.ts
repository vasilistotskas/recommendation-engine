/**
 * Carousel layout renderer
 */

import type { Product } from "../types";
import { renderCard } from "../templates/card";

export class CarouselWidget {
  private container: HTMLElement;
  private products: Product[];

  constructor(container: HTMLElement, products: Product[]) {
    this.container = container;
    this.products = products;
  }

  public render(): void {
    const html = `
      <div class="gs-carousel">
        <div class="gs-carousel-track">
          ${this.products.map((p) => renderCard(p)).join("")}
        </div>
        ${
          this.products.length > 3
            ? `
          <button class="gs-carousel-btn gs-carousel-prev" aria-label="Previous">
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path fill="currentColor" d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
            </svg>
          </button>
          <button class="gs-carousel-btn gs-carousel-next" aria-label="Next">
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path fill="currentColor" d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
            </svg>
          </button>
        `
            : ""
        }
      </div>
    `;

    this.container.innerHTML = html;
    this.attachEventHandlers();
  }

  private attachEventHandlers(): void {
    const track = this.container.querySelector(
      ".gs-carousel-track",
    ) as HTMLElement;
    const prevBtn = this.container.querySelector(
      ".gs-carousel-prev",
    ) as HTMLButtonElement;
    const nextBtn = this.container.querySelector(
      ".gs-carousel-next",
    ) as HTMLButtonElement;

    if (!track) return;

    let currentScroll = 0;
    const cardWidth = 280; // Approximate card width
    const scrollAmount = cardWidth * 2; // Scroll 2 cards at a time

    if (prevBtn) {
      prevBtn.addEventListener("click", () => {
        currentScroll = Math.max(0, currentScroll - scrollAmount);
        track.scrollTo({
          left: currentScroll,
          behavior: "smooth",
        });
      });
    }

    if (nextBtn) {
      nextBtn.addEventListener("click", () => {
        const maxScroll = track.scrollWidth - track.clientWidth;
        currentScroll = Math.min(maxScroll, currentScroll + scrollAmount);
        track.scrollTo({
          left: currentScroll,
          behavior: "smooth",
        });
      });
    }

    // Touch/swipe support
    let startX = 0;
    let isDragging = false;

    track.addEventListener("touchstart", (e) => {
      startX = e.touches[0].clientX;
      isDragging = true;
    });

    track.addEventListener("touchmove", (e) => {
      if (!isDragging) return;
      const currentX = e.touches[0].clientX;
      const diff = startX - currentX;

      if (Math.abs(diff) > 50) {
        if (diff > 0) {
          nextBtn?.click();
        } else {
          prevBtn?.click();
        }
        isDragging = false;
      }
    });

    track.addEventListener("touchend", () => {
      isDragging = false;
    });
  }
}
