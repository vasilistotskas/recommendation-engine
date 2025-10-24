<?php
/**
 * Gutenberg block handler
 */

if (!defined('ABSPATH')) {
    exit;
}

class GrooveShop_Block {

    /**
     * Register Gutenberg block
     */
    public static function register() {
        // Only register if Gutenberg is available
        if (!function_exists('register_block_type')) {
            return;
        }

        // Register block script
        wp_register_script(
            'grooveshop-block',
            GROOVESHOP_PLUGIN_URL . 'blocks/recommendations/block.js',
            array('wp-blocks', 'wp-element', 'wp-editor', 'wp-components'),
            GROOVESHOP_VERSION
        );

        // Register block
        register_block_type('grooveshop/recommendations', array(
            'editor_script' => 'grooveshop-block',
            'render_callback' => array('GrooveShop_Shortcode', 'render'),
            'attributes' => array(
                'productId' => array(
                    'type' => 'string',
                    'default' => '',
                ),
                'count' => array(
                    'type' => 'number',
                    'default' => 5,
                ),
                'type' => array(
                    'type' => 'string',
                    'default' => 'similar',
                ),
                'layout' => array(
                    'type' => 'string',
                    'default' => 'carousel',
                ),
                'theme' => array(
                    'type' => 'string',
                    'default' => 'light',
                ),
                'realTime' => array(
                    'type' => 'boolean',
                    'default' => false,
                ),
            ),
        ));
    }
}
