/**
 * Product card template
 */

import type { Product } from "../types";
import type { RealtimeData } from "../realtime";

export function renderCard(product: Product, realtime?: RealtimeData): string {
  const {
    entity_id,
    attributes: {
      name,
      price,
      image_url,
      rating,
      review_count,
      discount,
      stock,
    },
  } = product;

  const hasDiscount = discount && discount > 0;
  const discountedPrice = hasDiscount ? price * (1 - discount / 100) : price;
  const isLowStock = stock !== undefined && stock > 0 && stock <= 5;

  return `
    <div class="gs-card" data-product-id="${entity_id}">
      <div class="gs-card-image-wrapper">
        ${hasDiscount ? `<span class="gs-badge gs-badge-discount">-${discount}%</span>` : ""}
        ${isLowStock ? `<span class="gs-badge gs-badge-stock">Only ${stock} left</span>` : ""}
        <img
          src="${image_url || "/placeholder.png"}"
          alt="${escapeHtml(name)}"
          class="gs-card-image"
          loading="lazy"
        />
      </div>
      ${renderSocialProof(realtime)}
      <div class="gs-card-content">
        <h3 class="gs-card-title">${escapeHtml(name)}</h3>
        ${
          rating
            ? `
          <div class="gs-card-rating">
            <span class="gs-stars">${renderStars(rating)}</span>
            ${review_count ? `<span class="gs-review-count">(${review_count})</span>` : ""}
          </div>
        `
            : ""
        }
        <div class="gs-card-price">
          ${
            hasDiscount
              ? `
            <span class="gs-price-old">$${price.toFixed(2)}</span>
            <span class="gs-price-sale">$${discountedPrice.toFixed(2)}</span>
          `
              : `
            <span class="gs-price">$${price.toFixed(2)}</span>
          `
          }
        </div>
        <a href="/products/${entity_id}" class="gs-btn gs-btn-primary">
          View Product
        </a>
      </div>
    </div>
  `;
}

export function renderSkeleton(): string {
  return `
    <div class="gs-card gs-skeleton">
      <div class="gs-skeleton-image shimmer"></div>
      <div class="gs-skeleton-content">
        <div class="gs-skeleton-title shimmer"></div>
        <div class="gs-skeleton-rating shimmer"></div>
        <div class="gs-skeleton-price shimmer"></div>
        <div class="gs-skeleton-button shimmer"></div>
      </div>
    </div>
  `;
}

function renderStars(rating: number): string {
  const fullStars = Math.floor(rating);
  const hasHalfStar = rating % 1 >= 0.5;
  const emptyStars = 5 - fullStars - (hasHalfStar ? 1 : 0);

  return (
    "★".repeat(fullStars) + (hasHalfStar ? "☆" : "") + "☆".repeat(emptyStars)
  );
}

function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

/**
 * Render social proof badges for real-time data
 */
function renderSocialProof(realtime?: RealtimeData): string {
  if (!realtime) return "";

  const badges: string[] = [];

  // Viewing now badge (live pulse)
  if (realtime.viewingNow > 0) {
    badges.push(`
      <span class="gs-badge gs-badge-live">
        <span class="gs-pulse"></span>
        ${realtime.viewingNow} viewing now
      </span>
    `);
  }

  // Recent sales badge
  if (realtime.recentSales > 0) {
    badges.push(`
      <span class="gs-badge gs-badge-sales">
        ${realtime.recentSales} sold today
      </span>
    `);
  }

  // Added to cart badge
  if (realtime.addedToCart > 0) {
    badges.push(`
      <span class="gs-badge gs-badge-cart">
        ${realtime.addedToCart} in carts
      </span>
    `);
  }

  if (badges.length === 0) return "";

  return `
    <div class="gs-social-proof" data-product-id="${realtime.productId}">
      ${badges.join("")}
    </div>
  `;
}
