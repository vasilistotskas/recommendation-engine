/**
 * Real-time WebSocket Client for Live Social Proof
 *
 * Connects to the recommendation engine WebSocket endpoint to receive
 * real-time updates about product activity (views, purchases, etc.)
 */

export interface RealtimeData {
  productId: string;
  viewingNow: number;
  recentSales: number;
  recentViews: number;
  addedToCart: number;
  timestamp: number;
}

export type RealtimeEventType =
  | 'product:views'
  | 'product:sales'
  | 'product:cart'
  | 'connection:open'
  | 'connection:close'
  | 'connection:error';

export type RealtimeCallback = (data: RealtimeData) => void;

export interface RealtimeConfig {
  apiUrl: string;
  reconnectDelay?: number;
  maxReconnectAttempts?: number;
  debug?: boolean;
}

/**
 * WebSocket client for real-time product activity updates
 */
export class RealtimeClient {
  private ws: WebSocket | null = null;
  private listeners: Map<string, Set<RealtimeCallback>> = new Map();
  private reconnectAttempts = 0;
  private reconnectTimer: number | null = null;
  private isConnecting = false;
  private shouldReconnect = true;

  private readonly reconnectDelay: number;
  private readonly maxReconnectAttempts: number;
  private readonly debug: boolean;
  private readonly wsUrl: string;

  constructor(config: RealtimeConfig) {
    this.wsUrl = config.apiUrl.replace(/^http/, 'ws') + '/ws/activity';
    this.reconnectDelay = config.reconnectDelay || 3000;
    this.maxReconnectAttempts = config.maxReconnectAttempts || 5;
    this.debug = config.debug || false;
  }

  /**
   * Connect to WebSocket server
   */
  public connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN || this.isConnecting) {
      return;
    }

    this.isConnecting = true;
    this.log('Connecting to WebSocket:', this.wsUrl);

    try {
      this.ws = new WebSocket(this.wsUrl);
      this.setupEventHandlers();
    } catch (error) {
      this.error('Failed to create WebSocket connection:', error);
      this.isConnecting = false;
      this.scheduleReconnect();
    }
  }

  /**
   * Disconnect from WebSocket server
   */
  public disconnect(): void {
    this.shouldReconnect = false;

    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.log('Disconnected from WebSocket');
  }

  /**
   * Subscribe to real-time updates for a specific product
   */
  public subscribe(productId: string, callback: RealtimeCallback): () => void {
    const key = `product:${productId}`;

    if (!this.listeners.has(key)) {
      this.listeners.set(key, new Set());
    }

    this.listeners.get(key)!.add(callback);
    this.log(`Subscribed to product:${productId}`);

    // Send subscription message to server
    this.send({
      type: 'subscribe',
      productId,
    });

    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(key);
      if (listeners) {
        listeners.delete(callback);
        if (listeners.size === 0) {
          this.listeners.delete(key);
          this.send({
            type: 'unsubscribe',
            productId,
          });
        }
      }
    };
  }

  /**
   * Check if WebSocket is connected
   */
  public isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Setup WebSocket event handlers
   */
  private setupEventHandlers(): void {
    if (!this.ws) return;

    this.ws.onopen = () => {
      this.isConnecting = false;
      this.reconnectAttempts = 0;
      this.log('WebSocket connected');
      this.emit('connection:open', {
        productId: '',
        viewingNow: 0,
        recentSales: 0,
        recentViews: 0,
        addedToCart: 0,
        timestamp: Date.now(),
      });
    };

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleMessage(data);
      } catch (error) {
        this.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onerror = (event) => {
      this.error('WebSocket error:', event);
      this.emit('connection:error', {
        productId: '',
        viewingNow: 0,
        recentSales: 0,
        recentViews: 0,
        addedToCart: 0,
        timestamp: Date.now(),
      });
    };

    this.ws.onclose = () => {
      this.isConnecting = false;
      this.log('WebSocket closed');
      this.emit('connection:close', {
        productId: '',
        viewingNow: 0,
        recentSales: 0,
        recentViews: 0,
        addedToCart: 0,
        timestamp: Date.now(),
      });

      if (this.shouldReconnect) {
        this.scheduleReconnect();
      }
    };
  }

  /**
   * Handle incoming WebSocket message
   */
  private handleMessage(data: any): void {
    if (data.type === 'activity' && data.productId) {
      const realtimeData: RealtimeData = {
        productId: data.productId,
        viewingNow: data.viewingNow || 0,
        recentSales: data.recentSales || 0,
        recentViews: data.recentViews || 0,
        addedToCart: data.addedToCart || 0,
        timestamp: data.timestamp || Date.now(),
      };

      const key = `product:${data.productId}`;
      this.emit(key, realtimeData);
    }
  }

  /**
   * Emit event to all subscribed listeners
   */
  private emit(event: string, data: RealtimeData): void {
    const listeners = this.listeners.get(event);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          this.error('Error in listener callback:', error);
        }
      });
    }
  }

  /**
   * Send message to WebSocket server
   */
  private send(message: any): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(message));
      } catch (error) {
        this.error('Failed to send WebSocket message:', error);
      }
    } else {
      this.log('WebSocket not connected, queuing message');
      // TODO: Queue messages and send when connected
    }
  }

  /**
   * Schedule reconnection attempt
   */
  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      this.error('Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

    this.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);

    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
  }

  /**
   * Log debug message
   */
  private log(...args: any[]): void {
    if (this.debug) {
      console.log('[GrooveShop Realtime]', ...args);
    }
  }

  /**
   * Log error message
   */
  private error(...args: any[]): void {
    console.error('[GrooveShop Realtime]', ...args);
  }
}
