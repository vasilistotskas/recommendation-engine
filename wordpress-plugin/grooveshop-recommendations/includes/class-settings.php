<?php
/**
 * Settings handler
 */

if (!defined('ABSPATH')) {
    exit;
}

class GrooveShop_Settings {

    /**
     * Get all settings
     *
     * @return array Settings array
     */
    public static function get_all() {
        return array(
            'api_key' => get_option('grooveshop_api_key', ''),
            'tenant_id' => get_option('grooveshop_tenant_id', ''),
            'api_url' => get_option('grooveshop_api_url', 'https://api.grooveshop.com'),
            'auto_track' => get_option('grooveshop_auto_track', '1'),
            'enable_product_page' => get_option('grooveshop_enable_product_page', '1'),
            'enable_cart_page' => get_option('grooveshop_enable_cart_page', '1'),
            'layout' => get_option('grooveshop_layout', 'carousel'),
            'count' => get_option('grooveshop_count', '5'),
            'theme' => get_option('grooveshop_theme', 'light'),
        );
    }

    /**
     * Validate API credentials
     *
     * @param string $api_key API key
     * @param string $tenant_id Tenant ID
     * @return bool|WP_Error True if valid, WP_Error otherwise
     */
    public static function validate_credentials($api_key, $tenant_id) {
        if (empty($api_key) || empty($tenant_id)) {
            return new WP_Error('invalid_credentials', __('API Key and Tenant ID are required.', 'grooveshop-recommendations'));
        }

        // Test API connection
        $api_url = get_option('grooveshop_api_url', 'https://api.grooveshop.com');
        $response = wp_remote_get($api_url . '/api/v1/recommendations/' . $tenant_id . '/trending?count=1', array(
            'headers' => array(
                'Authorization' => 'Bearer ' . $api_key,
            ),
            'timeout' => 10,
        ));

        if (is_wp_error($response)) {
            return new WP_Error('api_error', __('Could not connect to GrooveShop API.', 'grooveshop-recommendations'));
        }

        $code = wp_remote_retrieve_response_code($response);
        if ($code !== 200) {
            return new WP_Error('invalid_credentials', __('Invalid API credentials.', 'grooveshop-recommendations'));
        }

        return true;
    }
}
