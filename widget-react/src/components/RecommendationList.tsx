/**
 * Recommendation List Component
 */

import React from 'react';
import { Recommendations } from './Recommendations';
import type { RecommendationProps } from '../types';

export interface RecommendationListProps extends Omit<RecommendationProps, 'layout'> {}

export const RecommendationList: React.FC<RecommendationListProps> = (props) => {
  return <Recommendations {...props} layout="list" />;
};
