<?php
/**
 * WooCommerce integration
 */

if (!defined('ABSPATH')) {
    exit;
}

class GrooveShop_WooCommerce {

    /**
     * Track order completion
     *
     * @param int $order_id Order ID
     */
    public static function track_order($order_id) {
        $api_key = get_option('grooveshop_api_key');
        $tenant_id = get_option('grooveshop_tenant_id');
        $api_url = get_option('grooveshop_api_url', 'https://api.grooveshop.com');

        if (empty($api_key) || empty($tenant_id)) {
            return;
        }

        $order = wc_get_order($order_id);
        if (!$order) {
            return;
        }

        $user_id = $order->get_user_id();
        $items = array();

        foreach ($order->get_items() as $item) {
            $product = $item->get_product();
            if (!$product) {
                continue;
            }

            $items[] = array(
                'entity_id' => $product->get_id(),
                'quantity' => $item->get_quantity(),
                'price' => $product->get_price(),
            );
        }

        // Send purchase event to API
        $data = array(
            'tenant_id' => $tenant_id,
            'user_id' => $user_id ? 'user_' . $user_id : 'guest_' . $order->get_billing_email(),
            'order_id' => $order_id,
            'items' => $items,
            'total' => $order->get_total(),
            'timestamp' => current_time('c'),
        );

        wp_remote_post($api_url . '/api/v1/interactions', array(
            'headers' => array(
                'Authorization' => 'Bearer ' . $api_key,
                'Content-Type' => 'application/json',
            ),
            'body' => json_encode($data),
            'timeout' => 15,
        ));
    }

    /**
     * Sync product to recommendation engine
     *
     * @param int $product_id Product ID
     */
    public static function sync_product($product_id) {
        $api_key = get_option('grooveshop_api_key');
        $tenant_id = get_option('grooveshop_tenant_id');
        $api_url = get_option('grooveshop_api_url', 'https://api.grooveshop.com');

        if (empty($api_key) || empty($tenant_id)) {
            return;
        }

        $product = wc_get_product($product_id);
        if (!$product) {
            return;
        }

        $data = array(
            'entity_id' => $product->get_id(),
            'attributes' => array(
                'name' => $product->get_name(),
                'price' => $product->get_price(),
                'image_url' => wp_get_attachment_image_url($product->get_image_id(), 'full'),
                'category' => self::get_product_categories($product),
                'stock' => $product->get_stock_quantity(),
                'rating' => $product->get_average_rating(),
                'review_count' => $product->get_review_count(),
            ),
        );

        wp_remote_post($api_url . '/api/v1/products/' . $tenant_id, array(
            'headers' => array(
                'Authorization' => 'Bearer ' . $api_key,
                'Content-Type' => 'application/json',
            ),
            'body' => json_encode($data),
            'timeout' => 15,
        ));
    }

    /**
     * Get product categories
     *
     * @param WC_Product $product Product object
     * @return string Comma-separated categories
     */
    private static function get_product_categories($product) {
        $terms = get_the_terms($product->get_id(), 'product_cat');
        if (!$terms || is_wp_error($terms)) {
            return '';
        }

        $categories = array();
        foreach ($terms as $term) {
            $categories[] = $term->name;
        }

        return implode(', ', $categories);
    }
}
