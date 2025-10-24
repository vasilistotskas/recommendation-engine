/**
 * List layout renderer
 */

import type { Product } from '../types';
import { renderCard } from '../templates/card';

export class ListWidget {
  private container: HTMLElement;
  private products: Product[];

  constructor(container: HTMLElement, products: Product[]) {
    this.container = container;
    this.products = products;
  }

  public render(): void {
    const html = `
      <div class="gs-list">
        ${this.products.map(p => renderCard(p)).join('')}
      </div>
    `;

    this.container.innerHTML = html;
  }
}
