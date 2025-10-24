/**
 * Recommendation Carousel Component
 */

import React from 'react';
import { Recommendations } from './Recommendations';
import type { RecommendationProps } from '../types';

export interface RecommendationCarouselProps extends Omit<RecommendationProps, 'layout'> {}

export const RecommendationCarousel: React.FC<RecommendationCarouselProps> = (props) => {
  return <Recommendations {...props} layout="carousel" />;
};
