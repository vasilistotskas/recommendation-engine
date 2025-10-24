<?php
/**
 * Settings page template
 */

if (!defined('ABSPATH')) {
    exit;
}

// Handle form submission
if (isset($_POST['grooveshop_save_settings']) && check_admin_referer('grooveshop_settings')) {
    // Save settings
    update_option('grooveshop_api_key', sanitize_text_field($_POST['grooveshop_api_key']));
    update_option('grooveshop_tenant_id', sanitize_text_field($_POST['grooveshop_tenant_id']));
    update_option('grooveshop_api_url', esc_url_raw($_POST['grooveshop_api_url']));
    update_option('grooveshop_auto_track', isset($_POST['grooveshop_auto_track']) ? '1' : '0');
    update_option('grooveshop_enable_product_page', isset($_POST['grooveshop_enable_product_page']) ? '1' : '0');
    update_option('grooveshop_enable_cart_page', isset($_POST['grooveshop_enable_cart_page']) ? '1' : '0');
    update_option('grooveshop_layout', sanitize_text_field($_POST['grooveshop_layout']));
    update_option('grooveshop_count', absint($_POST['grooveshop_count']));
    update_option('grooveshop_theme', sanitize_text_field($_POST['grooveshop_theme']));

    echo '<div class="notice notice-success"><p>' . esc_html__('Settings saved successfully!', 'grooveshop-recommendations') . '</p></div>';
}

$settings = GrooveShop_Settings::get_all();
?>

<div class="wrap grooveshop-settings">
    <h1><?php echo esc_html(get_admin_page_title()); ?></h1>

    <div class="grooveshop-settings-container">
        <div class="grooveshop-settings-main">
            <form method="post" action="">
                <?php wp_nonce_field('grooveshop_settings'); ?>

                <!-- API Settings -->
                <div class="grooveshop-card">
                    <h2><?php _e('API Settings', 'grooveshop-recommendations'); ?></h2>
                    <p class="description">
                        <?php _e('Enter your GrooveShop API credentials. Get your API key from', 'grooveshop-recommendations'); ?>
                        <a href="https://dashboard.grooveshop.com" target="_blank"><?php _e('your dashboard', 'grooveshop-recommendations'); ?></a>.
                    </p>

                    <table class="form-table">
                        <tr>
                            <th scope="row">
                                <label for="grooveshop_api_key"><?php _e('API Key', 'grooveshop-recommendations'); ?> *</label>
                            </th>
                            <td>
                                <input type="text"
                                       id="grooveshop_api_key"
                                       name="grooveshop_api_key"
                                       value="<?php echo esc_attr($settings['api_key']); ?>"
                                       class="regular-text"
                                       required>
                                <p class="description"><?php _e('Your public API key (starts with pk_)', 'grooveshop-recommendations'); ?></p>
                            </td>
                        </tr>
                        <tr>
                            <th scope="row">
                                <label for="grooveshop_tenant_id"><?php _e('Tenant ID', 'grooveshop-recommendations'); ?> *</label>
                            </th>
                            <td>
                                <input type="text"
                                       id="grooveshop_tenant_id"
                                       name="grooveshop_tenant_id"
                                       value="<?php echo esc_attr($settings['tenant_id']); ?>"
                                       class="regular-text"
                                       required>
                                <p class="description"><?php _e('Your store identifier', 'grooveshop-recommendations'); ?></p>
                            </td>
                        </tr>
                        <tr>
                            <th scope="row">
                                <label for="grooveshop_api_url"><?php _e('API URL', 'grooveshop-recommendations'); ?></label>
                            </th>
                            <td>
                                <input type="url"
                                       id="grooveshop_api_url"
                                       name="grooveshop_api_url"
                                       value="<?php echo esc_url($settings['api_url']); ?>"
                                       class="regular-text">
                                <p class="description"><?php _e('Leave default unless using custom endpoint', 'grooveshop-recommendations'); ?></p>
                            </td>
                        </tr>
                    </table>
                </div>

                <!-- Widget Settings -->
                <div class="grooveshop-card">
                    <h2><?php _e('Widget Settings', 'grooveshop-recommendations'); ?></h2>

                    <table class="form-table">
                        <tr>
                            <th scope="row"><?php _e('Auto Placement', 'grooveshop-recommendations'); ?></th>
                            <td>
                                <fieldset>
                                    <label>
                                        <input type="checkbox"
                                               name="grooveshop_enable_product_page"
                                               value="1"
                                               <?php checked($settings['enable_product_page'], '1'); ?>>
                                        <?php _e('Show similar products on product pages', 'grooveshop-recommendations'); ?>
                                    </label>
                                    <br>
                                    <label>
                                        <input type="checkbox"
                                               name="grooveshop_enable_cart_page"
                                               value="1"
                                               <?php checked($settings['enable_cart_page'], '1'); ?>>
                                        <?php _e('Show bundles on cart page', 'grooveshop-recommendations'); ?>
                                    </label>
                                </fieldset>
                            </td>
                        </tr>
                        <tr>
                            <th scope="row">
                                <label for="grooveshop_layout"><?php _e('Default Layout', 'grooveshop-recommendations'); ?></label>
                            </th>
                            <td>
                                <select id="grooveshop_layout" name="grooveshop_layout">
                                    <option value="carousel" <?php selected($settings['layout'], 'carousel'); ?>>
                                        <?php _e('Carousel', 'grooveshop-recommendations'); ?>
                                    </option>
                                    <option value="grid" <?php selected($settings['layout'], 'grid'); ?>>
                                        <?php _e('Grid', 'grooveshop-recommendations'); ?>
                                    </option>
                                    <option value="list" <?php selected($settings['layout'], 'list'); ?>>
                                        <?php _e('List', 'grooveshop-recommendations'); ?>
                                    </option>
                                </select>
                            </td>
                        </tr>
                        <tr>
                            <th scope="row">
                                <label for="grooveshop_count"><?php _e('Products to Show', 'grooveshop-recommendations'); ?></label>
                            </th>
                            <td>
                                <input type="number"
                                       id="grooveshop_count"
                                       name="grooveshop_count"
                                       value="<?php echo esc_attr($settings['count']); ?>"
                                       min="1"
                                       max="20"
                                       class="small-text">
                                <p class="description"><?php _e('Number of products to display (1-20)', 'grooveshop-recommendations'); ?></p>
                            </td>
                        </tr>
                        <tr>
                            <th scope="row">
                                <label for="grooveshop_theme"><?php _e('Theme', 'grooveshop-recommendations'); ?></label>
                            </th>
                            <td>
                                <select id="grooveshop_theme" name="grooveshop_theme">
                                    <option value="light" <?php selected($settings['theme'], 'light'); ?>>
                                        <?php _e('Light', 'grooveshop-recommendations'); ?>
                                    </option>
                                    <option value="dark" <?php selected($settings['theme'], 'dark'); ?>>
                                        <?php _e('Dark', 'grooveshop-recommendations'); ?>
                                    </option>
                                    <option value="minimal" <?php selected($settings['theme'], 'minimal'); ?>>
                                        <?php _e('Minimal', 'grooveshop-recommendations'); ?>
                                    </option>
                                </select>
                            </td>
                        </tr>
                        <tr>
                            <th scope="row"><?php _e('Tracking', 'grooveshop-recommendations'); ?></th>
                            <td>
                                <label>
                                    <input type="checkbox"
                                           name="grooveshop_auto_track"
                                           value="1"
                                           <?php checked($settings['auto_track'], '1'); ?>>
                                    <?php _e('Automatically track clicks and impressions', 'grooveshop-recommendations'); ?>
                                </label>
                            </td>
                        </tr>
                    </table>
                </div>

                <p class="submit">
                    <button type="submit" name="grooveshop_save_settings" class="button button-primary button-large">
                        <?php _e('Save Settings', 'grooveshop-recommendations'); ?>
                    </button>
                </p>
            </form>
        </div>

        <div class="grooveshop-settings-sidebar">
            <!-- Getting Started -->
            <div class="grooveshop-card">
                <h3><?php _e('Getting Started', 'grooveshop-recommendations'); ?></h3>
                <ol>
                    <li><?php _e('Enter your API credentials', 'grooveshop-recommendations'); ?></li>
                    <li><?php _e('Configure widget settings', 'grooveshop-recommendations'); ?></li>
                    <li><?php _e('Use shortcode or block to add widgets', 'grooveshop-recommendations'); ?></li>
                </ol>
            </div>

            <!-- Shortcode Usage -->
            <div class="grooveshop-card">
                <h3><?php _e('Shortcode Usage', 'grooveshop-recommendations'); ?></h3>
                <p><strong><?php _e('Basic:', 'grooveshop-recommendations'); ?></strong></p>
                <code>[grooveshop_recommendations]</code>

                <p><strong><?php _e('Similar Products:', 'grooveshop-recommendations'); ?></strong></p>
                <code>[grooveshop_recommendations type="similar" product_id="123"]</code>

                <p><strong><?php _e('Trending:', 'grooveshop-recommendations'); ?></strong></p>
                <code>[grooveshop_recommendations type="trending" count="8"]</code>

                <p><strong><?php _e('Bundle:', 'grooveshop-recommendations'); ?></strong></p>
                <code>[grooveshop_recommendations type="bundle"]</code>
            </div>

            <!-- Support -->
            <div class="grooveshop-card">
                <h3><?php _e('Need Help?', 'grooveshop-recommendations'); ?></h3>
                <p>
                    <a href="https://docs.grooveshop.com" target="_blank" class="button">
                        <?php _e('Documentation', 'grooveshop-recommendations'); ?>
                    </a>
                </p>
                <p>
                    <a href="https://support.grooveshop.com" target="_blank" class="button">
                        <?php _e('Support', 'grooveshop-recommendations'); ?>
                    </a>
                </p>
            </div>
        </div>
    </div>
</div>
