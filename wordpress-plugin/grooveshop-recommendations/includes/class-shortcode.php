<?php
/**
 * Shortcode handler
 */

if (!defined('ABSPATH')) {
    exit;
}

class GrooveShop_Shortcode {

    /**
     * Render shortcode
     *
     * @param array $atts Shortcode attributes
     * @return string HTML output
     */
    public static function render($atts) {
        $atts = shortcode_atts(array(
            'product_id' => '',
            'count' => get_option('grooveshop_count', '5'),
            'type' => 'similar',
            'layout' => get_option('grooveshop_layout', 'carousel'),
            'theme' => get_option('grooveshop_theme', 'light'),
            'real_time' => 'false',
        ), $atts, 'grooveshop_recommendations');

        // Auto-detect product ID if on product page
        if (empty($atts['product_id']) && function_exists('is_product') && is_product()) {
            global $product;
            if ($product) {
                $atts['product_id'] = $product->get_id();
            }
        }

        // Build data attributes
        $data_attrs = array(
            'data-grooveshop-recommendations' => '',
            'data-type' => esc_attr($atts['type']),
            'data-layout' => esc_attr($atts['layout']),
            'data-count' => esc_attr($atts['count']),
            'data-theme' => esc_attr($atts['theme']),
        );

        if (!empty($atts['product_id'])) {
            $data_attrs['data-product-id'] = esc_attr($atts['product_id']);
        }

        if ($atts['real_time'] === 'true') {
            $data_attrs['data-real-time'] = 'true';
        }

        // Get current user ID for personalized recommendations
        if ($atts['type'] === 'personalized' && is_user_logged_in()) {
            $data_attrs['data-user-id'] = get_current_user_id();
        }

        // Build HTML
        $html = '<div';
        foreach ($data_attrs as $key => $value) {
            $html .= ' ' . $key;
            if ($value !== '') {
                $html .= '="' . $value . '"';
            }
        }
        $html .= '></div>';

        return $html;
    }
}
