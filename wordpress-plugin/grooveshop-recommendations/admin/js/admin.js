/**
 * GrooveShop Recommendations Admin JavaScript
 */

(function($) {
    'use strict';

    $(document).ready(function() {
        // API validation
        let validationTimeout;
        const $apiKey = $('#grooveshop_api_key');
        const $tenantId = $('#grooveshop_tenant_id');
        const $apiUrl = $('#grooveshop_api_url');

        // Debounced validation
        function debounceValidation() {
            clearTimeout(validationTimeout);
            validationTimeout = setTimeout(validateCredentials, 1000);
        }

        $apiKey.on('input', debounceValidation);
        $tenantId.on('input', debounceValidation);
        $apiUrl.on('input', debounceValidation);

        // Validate credentials
        function validateCredentials() {
            const apiKey = $apiKey.val().trim();
            const tenantId = $tenantId.val().trim();
            const apiUrl = $apiUrl.val().trim() || 'https://api.grooveshop.com';

            // Remove existing status
            $('.grooveshop-api-status').remove();

            if (!apiKey || !tenantId) {
                return;
            }

            // Show checking status
            $apiKey.parent().append(
                '<div class="grooveshop-api-status checking">' +
                '<span class="grooveshop-spinner"></span>' +
                'Validating credentials...' +
                '</div>'
            );

            // Make test API call
            $.ajax({
                url: apiUrl + '/api/v1/recommendations/' + tenantId + '/trending?count=1',
                method: 'GET',
                headers: {
                    'Authorization': 'Bearer ' + apiKey
                },
                timeout: 10000,
                success: function() {
                    $('.grooveshop-api-status').remove();
                    $apiKey.parent().append(
                        '<div class="grooveshop-api-status valid">' +
                        '<span class="dashicons dashicons-yes-alt"></span>' +
                        'Credentials are valid!' +
                        '</div>'
                    );
                },
                error: function(xhr) {
                    $('.grooveshop-api-status').remove();
                    let message = 'Invalid credentials or connection error';

                    if (xhr.status === 401) {
                        message = 'Invalid API key';
                    } else if (xhr.status === 404) {
                        message = 'Invalid tenant ID';
                    } else if (xhr.status === 0) {
                        message = 'Cannot connect to API';
                    }

                    $apiKey.parent().append(
                        '<div class="grooveshop-api-status invalid">' +
                        '<span class="dashicons dashicons-dismiss"></span>' +
                        message +
                        '</div>'
                    );
                }
            });
        }

        // Preview widget
        const $previewBtn = $('<button type="button" class="button" style="margin-left: 10px;">Preview Widget</button>');
        $('.submit').prepend($previewBtn);

        $previewBtn.on('click', function(e) {
            e.preventDefault();
            showPreview();
        });

        function showPreview() {
            // Check if preview already exists
            if ($('.grooveshop-preview').length) {
                $('.grooveshop-preview').slideToggle();
                return;
            }

            // Get current settings
            const layout = $('#grooveshop_layout').val();
            const count = $('#grooveshop_count').val();
            const theme = $('#grooveshop_theme').val();

            // Create preview container
            const $preview = $('<div class="grooveshop-preview">' +
                '<h3>Widget Preview</h3>' +
                '<p class="description">This is a preview of how the widget will look with your current settings.</p>' +
                '<div class="grooveshop-preview-container">' +
                '<div data-grooveshop-recommendations ' +
                'data-type="trending" ' +
                'data-layout="' + layout + '" ' +
                'data-count="' + count + '" ' +
                'data-theme="' + theme + '">' +
                '</div>' +
                '</div>' +
                '</div>');

            $('.grooveshop-settings-main form').after($preview);
            $preview.slideDown();

            // Initialize widget if script is loaded
            if (window.GrooveShopRecommendations) {
                window.GrooveShopRecommendations.init();
            } else {
                $('.grooveshop-preview-container').html(
                    '<p style="text-align: center; padding: 40px; color: #646970;">' +
                    'Widget preview requires the GrooveShop widget script to be loaded on the frontend.' +
                    '</p>'
                );
            }
        }

        // Settings change handlers
        $('#grooveshop_layout, #grooveshop_count, #grooveshop_theme').on('change', function() {
            if ($('.grooveshop-preview').is(':visible')) {
                updatePreview();
            }
        });

        function updatePreview() {
            const layout = $('#grooveshop_layout').val();
            const count = $('#grooveshop_count').val();
            const theme = $('#grooveshop_theme').val();

            const $widget = $('.grooveshop-preview-container [data-grooveshop-recommendations]');
            $widget.attr('data-layout', layout);
            $widget.attr('data-count', count);
            $widget.attr('data-theme', theme);

            // Reinitialize widget
            if (window.GrooveShopRecommendations) {
                $widget.empty();
                window.GrooveShopRecommendations.init();
            }
        }

        // Auto-placement toggles
        $('#grooveshop_enable_product_page, #grooveshop_enable_cart_page').on('change', function() {
            const $checkbox = $(this);
            const isEnabled = $checkbox.is(':checked');
            const placement = $checkbox.attr('name') === 'grooveshop_enable_product_page' ? 'product pages' : 'cart page';

            if (isEnabled) {
                showNotice('Widget will automatically appear on ' + placement, 'info');
            }
        });

        // Helper to show notices
        function showNotice(message, type) {
            const $notice = $('<div class="notice notice-' + type + ' is-dismissible"><p>' + message + '</p></div>');
            $('.grooveshop-settings h1').after($notice);

            // Auto-dismiss after 3 seconds
            setTimeout(function() {
                $notice.fadeOut(function() {
                    $(this).remove();
                });
            }, 3000);

            // Make dismissible
            $notice.on('click', '.notice-dismiss', function() {
                $notice.fadeOut(function() {
                    $(this).remove();
                });
            });
        }

        // Form validation
        $('form').on('submit', function(e) {
            const apiKey = $apiKey.val().trim();
            const tenantId = $tenantId.val().trim();

            if (!apiKey || !tenantId) {
                e.preventDefault();
                showNotice('API Key and Tenant ID are required', 'error');
                return false;
            }

            // Validate API key format
            if (!apiKey.startsWith('pk_')) {
                e.preventDefault();
                showNotice('API Key should start with "pk_"', 'error');
                return false;
            }

            // Validate count range
            const count = parseInt($('#grooveshop_count').val());
            if (count < 1 || count > 20) {
                e.preventDefault();
                showNotice('Products to show must be between 1 and 20', 'error');
                return false;
            }
        });

        // Copy shortcode to clipboard
        $(document).on('click', '.grooveshop-settings-sidebar code', function() {
            const $code = $(this);
            const text = $code.text();

            // Create temporary textarea
            const $temp = $('<textarea>');
            $('body').append($temp);
            $temp.val(text).select();
            document.execCommand('copy');
            $temp.remove();

            // Show feedback
            const originalBg = $code.css('background-color');
            $code.css('background-color', '#d7f0db');
            setTimeout(function() {
                $code.css('background-color', originalBg);
            }, 200);

            showNotice('Shortcode copied to clipboard!', 'success');
        });

        // Validate on page load if credentials exist
        if ($apiKey.val() && $tenantId.val()) {
            validateCredentials();
        }
    });

})(jQuery);
