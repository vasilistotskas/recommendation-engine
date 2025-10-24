/**
 * GrooveShop Recommendations Gutenberg Block
 */

(function(wp) {
    const { registerBlockType } = wp.blocks;
    const { InspectorControls } = wp.blockEditor;
    const { PanelBody, SelectControl, TextControl, ToggleControl } = wp.components;
    const { Fragment } = wp.element;
    const { __ } = wp.i18n;

    registerBlockType('grooveshop/recommendations', {
        title: __('GrooveShop Recommendations', 'grooveshop-recommendations'),
        description: __('Display personalized product recommendations', 'grooveshop-recommendations'),
        icon: 'star-filled',
        category: 'widgets',
        keywords: [
            __('recommendations', 'grooveshop-recommendations'),
            __('products', 'grooveshop-recommendations'),
            __('grooveshop', 'grooveshop-recommendations')
        ],
        attributes: {
            productId: {
                type: 'string',
                default: ''
            },
            count: {
                type: 'number',
                default: 5
            },
            type: {
                type: 'string',
                default: 'similar'
            },
            layout: {
                type: 'string',
                default: 'carousel'
            },
            theme: {
                type: 'string',
                default: 'light'
            },
            realTime: {
                type: 'boolean',
                default: false
            }
        },

        edit: function(props) {
            const { attributes, setAttributes } = props;
            const { productId, count, type, layout, theme, realTime } = attributes;

            return (
                Fragment(null,
                    // Sidebar controls
                    InspectorControls(null,
                        PanelBody({
                            title: __('Recommendation Settings', 'grooveshop-recommendations'),
                            initialOpen: true
                        },
                            SelectControl({
                                label: __('Recommendation Type', 'grooveshop-recommendations'),
                                value: type,
                                options: [
                                    { label: __('Similar Products', 'grooveshop-recommendations'), value: 'similar' },
                                    { label: __('Trending Products', 'grooveshop-recommendations'), value: 'trending' },
                                    { label: __('Bundle', 'grooveshop-recommendations'), value: 'bundle' },
                                    { label: __('Personalized', 'grooveshop-recommendations'), value: 'personalized' },
                                    { label: __('Complement', 'grooveshop-recommendations'), value: 'complement' },
                                    { label: __('Recently Viewed', 'grooveshop-recommendations'), value: 'recently-viewed' },
                                    { label: __('Auto', 'grooveshop-recommendations'), value: 'auto' }
                                ],
                                onChange: function(value) {
                                    setAttributes({ type: value });
                                }
                            }),

                            (type === 'similar' || type === 'complement') && TextControl({
                                label: __('Product ID', 'grooveshop-recommendations'),
                                value: productId,
                                help: __('Leave empty to auto-detect on product pages', 'grooveshop-recommendations'),
                                onChange: function(value) {
                                    setAttributes({ productId: value });
                                }
                            }),

                            SelectControl({
                                label: __('Layout', 'grooveshop-recommendations'),
                                value: layout,
                                options: [
                                    { label: __('Carousel', 'grooveshop-recommendations'), value: 'carousel' },
                                    { label: __('Grid', 'grooveshop-recommendations'), value: 'grid' },
                                    { label: __('List', 'grooveshop-recommendations'), value: 'list' }
                                ],
                                onChange: function(value) {
                                    setAttributes({ layout: value });
                                }
                            }),

                            TextControl({
                                label: __('Number of Products', 'grooveshop-recommendations'),
                                type: 'number',
                                value: count,
                                min: 1,
                                max: 20,
                                onChange: function(value) {
                                    setAttributes({ count: parseInt(value) });
                                }
                            })
                        ),

                        PanelBody({
                            title: __('Appearance', 'grooveshop-recommendations'),
                            initialOpen: false
                        },
                            SelectControl({
                                label: __('Theme', 'grooveshop-recommendations'),
                                value: theme,
                                options: [
                                    { label: __('Light', 'grooveshop-recommendations'), value: 'light' },
                                    { label: __('Dark', 'grooveshop-recommendations'), value: 'dark' },
                                    { label: __('Minimal', 'grooveshop-recommendations'), value: 'minimal' }
                                ],
                                onChange: function(value) {
                                    setAttributes({ theme: value });
                                }
                            })
                        ),

                        PanelBody({
                            title: __('Advanced', 'grooveshop-recommendations'),
                            initialOpen: false
                        },
                            ToggleControl({
                                label: __('Real-time Updates', 'grooveshop-recommendations'),
                                help: __('Show live social proof and updates', 'grooveshop-recommendations'),
                                checked: realTime,
                                onChange: function(value) {
                                    setAttributes({ realTime: value });
                                }
                            })
                        )
                    ),

                    // Block preview
                    wp.element.createElement('div', {
                        className: 'grooveshop-block-preview',
                        style: {
                            padding: '20px',
                            background: '#f0f0f1',
                            border: '1px solid #ddd',
                            borderRadius: '4px',
                            textAlign: 'center'
                        }
                    },
                        wp.element.createElement('div', {
                            className: 'dashicons dashicons-star-filled',
                            style: {
                                fontSize: '48px',
                                width: '48px',
                                height: '48px',
                                marginBottom: '10px'
                            }
                        }),
                        wp.element.createElement('h3', {
                            style: { margin: '0 0 10px 0' }
                        }, __('GrooveShop Recommendations', 'grooveshop-recommendations')),
                        wp.element.createElement('p', {
                            style: { margin: '0 0 5px 0', color: '#666' }
                        }, __('Type:', 'grooveshop-recommendations') + ' ' + type),
                        wp.element.createElement('p', {
                            style: { margin: '0 0 5px 0', color: '#666' }
                        }, __('Layout:', 'grooveshop-recommendations') + ' ' + layout),
                        wp.element.createElement('p', {
                            style: { margin: '0', color: '#666' }
                        }, __('Products:', 'grooveshop-recommendations') + ' ' + count),
                        productId && wp.element.createElement('p', {
                            style: { margin: '5px 0 0 0', color: '#666', fontSize: '12px' }
                        }, __('Product ID:', 'grooveshop-recommendations') + ' ' + productId),
                        wp.element.createElement('div', {
                            style: {
                                marginTop: '15px',
                                padding: '10px',
                                background: '#fff',
                                borderRadius: '3px',
                                fontSize: '12px',
                                color: '#666'
                            }
                        }, __('Recommendations will appear here on the frontend', 'grooveshop-recommendations'))
                    )
                )
            );
        },

        save: function() {
            // Rendered on PHP side
            return null;
        }
    });

})(window.wp);
