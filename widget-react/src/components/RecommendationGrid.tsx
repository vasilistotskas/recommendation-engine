/**
 * Recommendation Grid Component
 */

import React from 'react';
import { Recommendations } from './Recommendations';
import type { RecommendationProps } from '../types';

export interface RecommendationGridProps extends Omit<RecommendationProps, 'layout'> {}

export const RecommendationGrid: React.FC<RecommendationGridProps> = (props) => {
  return <Recommendations {...props} layout="grid" />;
};
