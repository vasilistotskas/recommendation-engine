<?php
/**
 * Plugin Name: GrooveShop Recommendations
 * Plugin URI: https://grooveshop.com/wordpress-plugin
 * Description: AI-powered product recommendations for your WooCommerce store. Increase sales with personalized recommendations.
 * Version: 1.0.0
 * Author: GrooveShop
 * Author URI: https://grooveshop.com
 * License: GPL v2 or later
 * License URI: https://www.gnu.org/licenses/gpl-2.0.html
 * Text Domain: grooveshop-recommendations
 * Domain Path: /languages
 * Requires at least: 5.8
 * Requires PHP: 7.4
 * WC requires at least: 5.0
 * WC tested up to: 8.0
 */

// Exit if accessed directly
if (!defined('ABSPATH')) {
    exit;
}

// Define plugin constants
define('GROOVESHOP_VERSION', '1.0.0');
define('GROOVESHOP_PLUGIN_DIR', plugin_dir_path(__FILE__));
define('GROOVESHOP_PLUGIN_URL', plugin_dir_url(__FILE__));
define('GROOVESHOP_PLUGIN_BASENAME', plugin_basename(__FILE__));

/**
 * Main GrooveShop Recommendations class
 */
class GrooveShop_Recommendations {

    /**
     * Single instance of the class
     */
    private static $instance = null;

    /**
     * Get single instance
     */
    public static function get_instance() {
        if (self::$instance === null) {
            self::$instance = new self();
        }
        return self::$instance;
    }

    /**
     * Constructor
     */
    private function __construct() {
        $this->init_hooks();
        $this->includes();
    }

    /**
     * Initialize hooks
     */
    private function init_hooks() {
        // Activation/Deactivation hooks
        register_activation_hook(__FILE__, array($this, 'activate'));
        register_deactivation_hook(__FILE__, array($this, 'deactivate'));

        // Admin hooks
        add_action('admin_menu', array($this, 'add_admin_menu'));
        add_action('admin_init', array($this, 'register_settings'));
        add_action('admin_enqueue_scripts', array($this, 'admin_enqueue_scripts'));

        // Frontend hooks
        add_action('wp_enqueue_scripts', array($this, 'enqueue_scripts'));
        add_action('wp_head', array($this, 'add_widget_config'));

        // Shortcode
        add_shortcode('grooveshop_recommendations', array($this, 'shortcode'));

        // Gutenberg block
        add_action('init', array($this, 'register_block'));

        // WooCommerce hooks
        if (class_exists('WooCommerce')) {
            add_action('woocommerce_after_single_product_summary', array($this, 'add_similar_products'), 15);
            add_action('woocommerce_after_cart', array($this, 'add_bundle_recommendations'));
            add_action('woocommerce_order_status_completed', array($this, 'track_order'));
        }
    }

    /**
     * Include required files
     */
    private function includes() {
        require_once GROOVESHOP_PLUGIN_DIR . 'includes/class-settings.php';
        require_once GROOVESHOP_PLUGIN_DIR . 'includes/class-shortcode.php';
        require_once GROOVESHOP_PLUGIN_DIR . 'includes/class-block.php';
        require_once GROOVESHOP_PLUGIN_DIR . 'includes/class-woocommerce.php';
    }

    /**
     * Plugin activation
     */
    public function activate() {
        // Set default options
        add_option('grooveshop_api_key', '');
        add_option('grooveshop_tenant_id', '');
        add_option('grooveshop_api_url', 'https://api.grooveshop.com');
        add_option('grooveshop_auto_track', '1');
        add_option('grooveshop_enable_product_page', '1');
        add_option('grooveshop_enable_cart_page', '1');
        add_option('grooveshop_layout', 'carousel');
        add_option('grooveshop_count', '5');

        flush_rewrite_rules();
    }

    /**
     * Plugin deactivation
     */
    public function deactivate() {
        flush_rewrite_rules();
    }

    /**
     * Add admin menu
     */
    public function add_admin_menu() {
        add_menu_page(
            __('GrooveShop Recommendations', 'grooveshop-recommendations'),
            __('GrooveShop', 'grooveshop-recommendations'),
            'manage_options',
            'grooveshop-recommendations',
            array($this, 'render_settings_page'),
            'dashicons-chart-line',
            56
        );
    }

    /**
     * Register settings
     */
    public function register_settings() {
        // API Settings
        register_setting('grooveshop_settings', 'grooveshop_api_key');
        register_setting('grooveshop_settings', 'grooveshop_tenant_id');
        register_setting('grooveshop_settings', 'grooveshop_api_url');

        // Widget Settings
        register_setting('grooveshop_settings', 'grooveshop_auto_track');
        register_setting('grooveshop_settings', 'grooveshop_enable_product_page');
        register_setting('grooveshop_settings', 'grooveshop_enable_cart_page');
        register_setting('grooveshop_settings', 'grooveshop_layout');
        register_setting('grooveshop_settings', 'grooveshop_count');
        register_setting('grooveshop_settings', 'grooveshop_theme');
    }

    /**
     * Render settings page
     */
    public function render_settings_page() {
        include GROOVESHOP_PLUGIN_DIR . 'admin/settings-page.php';
    }

    /**
     * Enqueue admin scripts
     */
    public function admin_enqueue_scripts($hook) {
        if ($hook !== 'toplevel_page_grooveshop-recommendations') {
            return;
        }

        wp_enqueue_style(
            'grooveshop-admin',
            GROOVESHOP_PLUGIN_URL . 'admin/css/admin.css',
            array(),
            GROOVESHOP_VERSION
        );

        wp_enqueue_script(
            'grooveshop-admin',
            GROOVESHOP_PLUGIN_URL . 'admin/js/admin.js',
            array('jquery'),
            GROOVESHOP_VERSION,
            true
        );
    }

    /**
     * Enqueue frontend scripts
     */
    public function enqueue_scripts() {
        $api_key = get_option('grooveshop_api_key');

        if (empty($api_key)) {
            return;
        }

        // Enqueue widget script
        wp_enqueue_script(
            'grooveshop-widget',
            'https://cdn.grooveshop.com/widget/v1/widget.js',
            array(),
            GROOVESHOP_VERSION,
            true
        );

        // Enqueue widget styles
        wp_enqueue_style(
            'grooveshop-widget',
            'https://cdn.grooveshop.com/widget/v1/widget.css',
            array(),
            GROOVESHOP_VERSION
        );
    }

    /**
     * Add widget configuration to head
     */
    public function add_widget_config() {
        $api_key = get_option('grooveshop_api_key');
        $tenant_id = get_option('grooveshop_tenant_id');
        $api_url = get_option('grooveshop_api_url');
        $auto_track = get_option('grooveshop_auto_track', '1');

        if (empty($api_key) || empty($tenant_id)) {
            return;
        }
        ?>
        <script>
            window.GrooveShopConfig = {
                apiKey: '<?php echo esc_js($api_key); ?>',
                tenantId: '<?php echo esc_js($tenant_id); ?>',
                apiUrl: '<?php echo esc_js($api_url); ?>',
                autoTrack: <?php echo $auto_track === '1' ? 'true' : 'false'; ?>,
                debug: <?php echo defined('WP_DEBUG') && WP_DEBUG ? 'true' : 'false'; ?>
            };
        </script>
        <?php
    }

    /**
     * Shortcode handler
     */
    public function shortcode($atts) {
        return GrooveShop_Shortcode::render($atts);
    }

    /**
     * Register Gutenberg block
     */
    public function register_block() {
        GrooveShop_Block::register();
    }

    /**
     * Add similar products on product page
     */
    public function add_similar_products() {
        if (get_option('grooveshop_enable_product_page') !== '1') {
            return;
        }

        global $product;
        $product_id = $product ? $product->get_id() : null;

        if (!$product_id) {
            return;
        }

        echo '<div class="grooveshop-similar-products">';
        echo '<h2>' . esc_html__('You May Also Like', 'grooveshop-recommendations') . '</h2>';
        echo do_shortcode('[grooveshop_recommendations product_id="' . $product_id . '" type="similar"]');
        echo '</div>';
    }

    /**
     * Add bundle recommendations on cart page
     */
    public function add_bundle_recommendations() {
        if (get_option('grooveshop_enable_cart_page') !== '1') {
            return;
        }

        echo '<div class="grooveshop-bundle-recommendations">';
        echo '<h2>' . esc_html__('Frequently Bought Together', 'grooveshop-recommendations') . '</h2>';
        echo do_shortcode('[grooveshop_recommendations type="bundle"]');
        echo '</div>';
    }

    /**
     * Track order completion
     */
    public function track_order($order_id) {
        GrooveShop_WooCommerce::track_order($order_id);
    }
}

/**
 * Initialize the plugin
 */
function grooveshop_recommendations_init() {
    return GrooveShop_Recommendations::get_instance();
}

// Initialize
add_action('plugins_loaded', 'grooveshop_recommendations_init');
