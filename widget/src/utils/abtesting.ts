/**
 * A/B Testing System
 *
 * Allows testing different widget configurations with traffic splitting
 */

import type { WidgetAttributes } from "../types";

export interface ABVariant {
  name: string;
  weight: number; // Percentage 0-100
  config: Partial<WidgetAttributes>;
}

export interface ABTestConfig {
  testId: string;
  variants: ABVariant[];
}

export class ABTest {
  private variant: ABVariant;
  private userId: string;

  constructor(
    private testId: string,
    private variants: ABVariant[],
  ) {
    // Validate variants
    this.validateVariants();

    // Get or create user ID for consistent assignment
    this.userId = this.getUserId();

    // Assign variant based on user ID hash
    this.variant = this.assignVariant();
  }

  /**
   * Get the assigned variant configuration
   */
  public getConfig(): Partial<WidgetAttributes> {
    return this.variant.config;
  }

  /**
   * Get the assigned variant name
   */
  public getVariantName(): string {
    return this.variant.name;
  }

  /**
   * Track event for this A/B test
   */
  public track(eventType: string, metadata?: Record<string, any>): void {
    const event = {
      test_id: this.testId,
      variant: this.variant.name,
      user_id: this.userId,
      event_type: eventType,
      timestamp: Date.now(),
      metadata: metadata || {},
    };

    // Send to analytics endpoint
    this.sendEvent(event);

    // Log for debugging
    console.log("[A/B Test]", event);
  }

  /**
   * Validate variant configuration
   */
  private validateVariants(): void {
    if (this.variants.length === 0) {
      throw new Error("At least one variant is required");
    }

    const totalWeight = this.variants.reduce((sum, v) => sum + v.weight, 0);
    if (Math.abs(totalWeight - 100) > 0.01) {
      throw new Error(`Variant weights must sum to 100, got ${totalWeight}`);
    }
  }

  /**
   * Assign variant based on user ID hash
   * Uses consistent hashing to ensure same user always gets same variant
   */
  private assignVariant(): ABVariant {
    const hash = this.hashUserId(this.userId, this.testId);
    const bucket = hash % 100; // 0-99

    let cumulative = 0;
    for (const variant of this.variants) {
      cumulative += variant.weight;
      if (bucket < cumulative) {
        return variant;
      }
    }

    // Fallback to first variant (should never happen with valid weights)
    return this.variants[0];
  }

  /**
   * Hash user ID to get consistent bucket assignment
   */
  private hashUserId(userId: string, testId: string): number {
    const str = `${userId}-${testId}`;
    let hash = 0;

    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }

    return Math.abs(hash);
  }

  /**
   * Get or create user ID
   */
  private getUserId(): string {
    const key = "gs_ab_user_id";
    let userId = localStorage.getItem(key);

    if (!userId) {
      userId = `ab_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      localStorage.setItem(key, userId);
    }

    return userId;
  }

  /**
   * Send event to analytics
   */
  private sendEvent(event: any): void {
    // Store events in localStorage for later sending
    const key = "gs_ab_events";
    try {
      const events = JSON.parse(localStorage.getItem(key) || "[]");
      events.push(event);

      // Keep only last 100 events
      const trimmed = events.slice(-100);
      localStorage.setItem(key, JSON.stringify(trimmed));

      // Send to server (async, fire-and-forget)
      this.sendToServer(event);
    } catch (error) {
      console.error("[A/B Test] Failed to store event:", error);
    }
  }

  /**
   * Send event to server
   */
  private async sendToServer(event: any): Promise<void> {
    try {
      // Get API URL from config
      const apiUrl =
        (window as any).GrooveShopConfig?.apiUrl || "http://localhost:8080";

      await fetch(`${apiUrl}/api/v1/ab-events`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(event),
      });
    } catch (error) {
      // Fail silently - events are queued in localStorage
      console.error("[A/B Test] Failed to send event:", error);
    }
  }
}

/**
 * A/B Test Manager
 * Manages multiple A/B tests
 */
export class ABTestManager {
  private tests: Map<string, ABTest> = new Map();

  /**
   * Register a new A/B test
   */
  public register(config: ABTestConfig): ABTest {
    const test = new ABTest(config.testId, config.variants);
    this.tests.set(config.testId, test);
    return test;
  }

  /**
   * Get a registered A/B test
   */
  public get(testId: string): ABTest | undefined {
    return this.tests.get(testId);
  }

  /**
   * Check if a test is registered
   */
  public has(testId: string): boolean {
    return this.tests.has(testId);
  }
}
