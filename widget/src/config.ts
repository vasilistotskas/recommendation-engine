/**
 * Configuration loader
 */

import type { WidgetConfig } from './types';

export class Config {
  private config: WidgetConfig;

  constructor() {
    this.config = this.loadFromScriptTag();
  }

  private loadFromScriptTag(): WidgetConfig {
    const scriptTag = document.currentScript as HTMLScriptElement;

    if (!scriptTag) {
      throw new Error('GrooveShop: Could not find script tag');
    }

    const apiKey = scriptTag.dataset.apiKey;
    const tenantId = scriptTag.dataset.tenantId;

    if (!apiKey || !tenantId) {
      throw new Error('GrooveShop: Missing required attributes data-api-key and data-tenant-id');
    }

    return {
      apiKey,
      tenantId,
      apiUrl: scriptTag.dataset.apiUrl || 'http://localhost:8080',
      autoTrack: scriptTag.dataset.autoTrack !== 'false',
      debug: scriptTag.dataset.debug === 'true',
    };
  }

  public get(): WidgetConfig {
    return this.config;
  }

  public log(...args: any[]): void {
    if (this.config.debug) {
      console.log('[GrooveShop]', ...args);
    }
  }

  public error(...args: any[]): void {
    console.error('[GrooveShop]', ...args);
  }
}
