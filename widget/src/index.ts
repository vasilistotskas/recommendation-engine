/**
 * GrooveShop Recommendations Widget
 * Main entry point
 */

import "./styles/base.css";
import { Config } from "./config";
import { ApiClient } from "./api";
import type { WidgetAttributes, Product, TrackingEvent } from "./types";
import { CarouselWidget } from "./widgets/carousel";
import { GridWidget } from "./widgets/grid";
import { ListWidget } from "./widgets/list";
import { renderSkeleton } from "./templates/card";
import { RealtimeClient, type RealtimeData } from "./realtime";
import { LazyLoader, setupPrefetchOnHover } from "./utils/lazyload";
import { ABTestManager } from "./utils/abtesting";
import { AnalyticsIntegration } from "./utils/analytics";

class GrooveShopRecommendations {
  private config!: Config;
  private api!: ApiClient;
  private realtime!: RealtimeClient;
  private lazyLoader!: LazyLoader;
  private abTestManager!: ABTestManager;
  private analytics!: AnalyticsIntegration;

  constructor() {
    try {
      this.config = new Config();
      this.api = new ApiClient(this.config);

      // Initialize realtime client if enabled
      const cfg = this.config.get();
      this.realtime = new RealtimeClient({
        apiUrl: cfg.apiUrl || "http://localhost:8080",
        debug: cfg.debug,
      });

      // Connect to WebSocket for real-time updates
      this.realtime.connect();

      // Initialize lazy loader
      this.lazyLoader = new LazyLoader();

      // Initialize A/B test manager
      this.abTestManager = new ABTestManager();
      this.registerABTests();

      // Initialize analytics
      this.analytics = new AnalyticsIntegration();

      this.init();
    } catch (error) {
      console.error("GrooveShop Widget initialization failed:", error);
    }
  }

  /**
   * Initialize widget
   */
  private init(): void {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", () => this.renderAll());
    } else {
      this.renderAll();
    }

    this.config.log("Widget initialized");
  }

  /**
   * Find and render all widgets on the page
   */
  private renderAll(): void {
    const widgets = document.querySelectorAll<HTMLElement>(
      "[data-grooveshop-recommendations]",
    );

    this.config.log(`Found ${widgets.length} widget(s)`);

    widgets.forEach((element) => {
      this.renderWidget(element);
    });
  }

  /**
   * Register A/B tests from configuration
   */
  private registerABTests(): void {
    const cfg = this.config.get();
    if (!cfg.abTests) return;

    Object.entries(cfg.abTests).forEach(([testId, config]) => {
      this.abTestManager.register({
        testId,
        variants: config.variants,
      });
      this.config.log("Registered A/B test:", testId);
    });
  }

  /**
   * Render a single widget
   */
  private async renderWidget(element: HTMLElement): Promise<void> {
    let attributes = this.getAttributes(element);

    // Apply A/B test variant if specified
    if (attributes.testId) {
      const test = this.abTestManager.get(attributes.testId);
      if (test) {
        // Merge variant config with attributes
        attributes = { ...attributes, ...test.getConfig() };

        // Track impression
        test.track("impression");

        this.config.log("A/B test variant:", test.getVariantName());

        // Track clicks later
        element.addEventListener("click", () => {
          test.track("click");
        });
      } else {
        this.config.error("A/B test not found:", attributes.testId);
      }
    }

    // Show loading skeleton
    element.innerHTML = this.getSkeletonHTML(attributes);
    element.classList.add("gs-recommendations");

    try {
      // Fetch recommendations
      const products = await this.fetchRecommendations(attributes);

      if (products.length === 0) {
        this.config.log("No recommendations found, hiding widget");
        element.innerHTML = "";
        return;
      }

      // Render layout
      this.renderLayout(element, attributes.layout || "grid", products);

      // Track impression
      if (this.config.get().autoTrack) {
        this.trackImpression(attributes.productId, products);
      }

      // Attach click handlers
      this.attachClickHandlers(element, attributes.productId, products);

      // Subscribe to real-time updates if enabled
      if (attributes.realTime && this.realtime.isConnected()) {
        this.subscribeToRealtimeUpdates(element, products);
      }

      // Setup lazy loading for images
      this.lazyLoader.observe(element);

      // Setup prefetch on hover
      setupPrefetchOnHover(element);

      this.config.log("Widget rendered successfully");
    } catch (error) {
      this.config.error("Failed to render widget:", error);
      element.innerHTML = ""; // Fail silently
    }
  }

  /**
   * Get widget attributes from element
   */
  private getAttributes(element: HTMLElement): WidgetAttributes {
    return {
      productId: element.dataset.productId,
      count: parseInt(element.dataset.count || "5", 10),
      layout: (element.dataset.layout as any) || "grid",
      type: (element.dataset.type as any) || "similar",
      theme: (element.dataset.theme as any) || "light",
      realTime: element.dataset.realTime === "true",
      userId: element.dataset.userId,
      testId: element.dataset.testId,
    };
  }

  /**
   * Fetch recommendations based on type
   */
  private async fetchRecommendations(
    attributes: WidgetAttributes,
  ): Promise<Product[]> {
    const { type, productId, count = 5, userId } = attributes;

    switch (type) {
      case "similar":
        if (!productId) {
          this.config.error("Product ID required for similar recommendations");
          return [];
        }
        return this.api.fetchSimilar(productId, count);

      case "trending":
        return this.api.fetchTrending(count);

      case "bundle":
        const cartIds = this.api.getCartProductIds();
        if (cartIds.length === 0 && productId) {
          // Fallback to single product bundle
          return this.api.fetchBundle([productId], count);
        }
        return this.api.fetchBundle(cartIds, count);

      case "personalized":
        return this.api.fetchPersonalized(userId, count);

      case "complement":
        if (!productId) {
          this.config.error(
            "Product ID required for complement recommendations",
          );
          return [];
        }
        return this.api.fetchComplement(productId, count);

      case "recently-viewed":
        return this.api.fetchRecentlyViewed(count);

      case "auto":
        return this.autoDetectAndFetch(count);

      default:
        this.config.error(`Unknown recommendation type: ${type}`);
        return [];
    }
  }

  /**
   * Auto-detect context and fetch appropriate recommendations
   */
  private async autoDetectAndFetch(count: number): Promise<Product[]> {
    // Try to detect product ID from URL or meta tags
    const productId = this.detectProductId();

    if (productId) {
      this.config.log("Auto-detected product ID:", productId);
      return this.api.fetchSimilar(productId, count);
    }

    // Fallback to trending
    this.config.log("No product ID detected, showing trending");
    return this.api.fetchTrending(count);
  }

  /**
   * Detect product ID from page context
   */
  private detectProductId(): string | null {
    // Check URL patterns
    const urlPatterns = [
      /\/products\/(\d+)/,
      /\/product\/(\d+)/,
      /product_id=(\d+)/,
      /productId=(\d+)/,
    ];

    const url = window.location.href;
    for (const pattern of urlPatterns) {
      const match = url.match(pattern);
      if (match) {
        return match[1];
      }
    }

    // Check meta tags
    const metaSelectors = [
      'meta[property="product:id"]',
      'meta[name="product:id"]',
      'meta[property="og:product:id"]',
    ];

    for (const selector of metaSelectors) {
      const meta = document.querySelector(selector);
      if (meta) {
        return meta.getAttribute("content");
      }
    }

    // Check schema.org structured data
    const scripts = document.querySelectorAll(
      'script[type="application/ld+json"]',
    );
    for (const script of Array.from(scripts)) {
      try {
        const data = JSON.parse(script.textContent || "");
        if (data["@type"] === "Product" && data.productID) {
          return data.productID;
        }
      } catch {
        // Ignore parse errors
      }
    }

    return null;
  }

  /**
   * Render layout
   */
  private renderLayout(
    container: HTMLElement,
    layout: string,
    products: Product[],
  ): void {
    switch (layout) {
      case "carousel":
        new CarouselWidget(container, products).render();
        break;

      case "grid":
        new GridWidget(container, products).render();
        break;

      case "list":
        new ListWidget(container, products).render();
        break;

      default:
        new GridWidget(container, products).render();
    }
  }

  /**
   * Get skeleton HTML
   */
  private getSkeletonHTML(attributes: WidgetAttributes): string {
    const count = attributes.count || 5;
    const layout = attributes.layout || "grid";

    const skeletons = Array(count)
      .fill(null)
      .map(() => renderSkeleton())
      .join("");

    if (layout === "carousel") {
      return `
        <div class="gs-carousel">
          <div class="gs-carousel-track">${skeletons}</div>
        </div>
      `;
    }

    if (layout === "list") {
      return `<div class="gs-list">${skeletons}</div>`;
    }

    return `<div class="gs-grid">${skeletons}</div>`;
  }

  /**
   * Track impression
   */
  private trackImpression(
    sourceProductId: string | undefined,
    products: Product[],
  ): void {
    if (products.length === 0) return;

    // Use IntersectionObserver to track when widget is visible
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            products.forEach((product) => {
              this.track({
                type: "impression",
                sourceProductId,
                targetProductId: product.entity_id,
                timestamp: Date.now(),
              });
            });

            // Stop observing after first impression
            observer.disconnect();
          }
        });
      },
      { threshold: 0.5 },
    );

    const container = document.querySelector(".gs-recommendations");
    if (container) {
      observer.observe(container);
    }
  }

  /**
   * Attach click handlers
   */
  private attachClickHandlers(
    container: HTMLElement,
    sourceProductId: string | undefined,
    products: Product[],
  ): void {
    const cards = container.querySelectorAll<HTMLElement>(".gs-card");

    cards.forEach((card, index) => {
      card.addEventListener("click", () => {
        const productId = card.dataset.productId;
        if (!productId) return;

        const product = products[index];

        this.track({
          type: "click",
          sourceProductId,
          targetProductId: productId,
          timestamp: Date.now(),
          metadata: {
            position: index,
            score: product.score,
          },
        });
      });
    });
  }

  /**
   * Subscribe to real-time updates for products
   */
  private subscribeToRealtimeUpdates(
    container: HTMLElement,
    products: Product[],
  ): void {
    products.forEach((product) => {
      this.realtime.subscribe(product.entity_id, (data: RealtimeData) => {
        this.updateSocialProof(container, product.entity_id, data);
      });
    });
  }

  /**
   * Update social proof badges with new real-time data
   */
  private updateSocialProof(
    container: HTMLElement,
    productId: string,
    data: RealtimeData,
  ): void {
    const card = container.querySelector<HTMLElement>(
      `[data-product-id="${productId}"]`,
    );
    if (!card) return;

    // Find or create social proof container
    let proofDiv = card.querySelector<HTMLElement>(".gs-social-proof");

    if (!proofDiv) {
      // Create social proof div if it doesn't exist
      proofDiv = document.createElement("div");
      proofDiv.className = "gs-social-proof";
      proofDiv.dataset.productId = productId;

      // Insert after image wrapper
      const imageWrapper = card.querySelector(".gs-card-image-wrapper");
      if (imageWrapper && imageWrapper.nextSibling) {
        imageWrapper.parentNode?.insertBefore(
          proofDiv,
          imageWrapper.nextSibling,
        );
      }
    }

    // Build badges HTML
    const badges: string[] = [];

    if (data.viewingNow > 0) {
      badges.push(`
        <span class="gs-badge gs-badge-live">
          <span class="gs-pulse"></span>
          ${data.viewingNow} viewing now
        </span>
      `);
    }

    if (data.recentSales > 0) {
      badges.push(`
        <span class="gs-badge gs-badge-sales">
          ${data.recentSales} sold today
        </span>
      `);
    }

    if (data.addedToCart > 0) {
      badges.push(`
        <span class="gs-badge gs-badge-cart">
          ${data.addedToCart} in carts
        </span>
      `);
    }

    // Update the DOM
    proofDiv.innerHTML = badges.join("");

    // Hide if no badges
    if (badges.length === 0) {
      proofDiv.style.display = "none";
    } else {
      proofDiv.style.display = "flex";
    }

    this.config.log("Updated social proof for product:", productId, data);
  }

  /**
   * Track event
   */
  private track(event: TrackingEvent): void {
    this.config.log("Tracking event:", event);

    // Track to our API
    this.api.trackInteraction(event);

    // Track to analytics integrations
    this.analytics.track({
      type: event.type,
      productId: event.targetProductId,
      sourceProductId: event.sourceProductId,
      metadata: event.metadata,
    });
  }

  /**
   * Expose analytics API for custom event listeners
   */
  public on(eventType: string, callback: (event: any) => void): () => void {
    return this.analytics.on(eventType, callback);
  }
}

// Auto-initialize when script loads
if (typeof window !== "undefined") {
  (window as any).GrooveShopRecommendations = new GrooveShopRecommendations();
}

export default GrooveShopRecommendations;
