"use client";

import React, { FC, useEffect, useState } from "react";

export interface PercentageSplitInputProps {
  amount: number;
  symbol: string | undefined;
  weight: number;
  onCommitWeight: (newWeight: number) => void;
}

export const PercentageSplitInput: FC<PercentageSplitInputProps> = ({
  amount,
  symbol,
  weight,
  onCommitWeight,
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const [localValue, setLocalValue] = useState(weight.toString());

  useEffect(() => {
    if (!isFocused) {
      setLocalValue(weight.toString());
    }
  }, [weight, isFocused]);

  const commit = () => {
    const parsed = parseFloat(localValue);
    if (!isNaN(parsed)) {
      const clampedWeight = Math.min(100, Math.max(0, parsed));
      onCommitWeight(clampedWeight);
    } else {
      setLocalValue(weight.toString());
    }
  };

  return (
    <div className="flex flex-col items-end gap-0.5">
      <div className="relative">
        <input
          type="number"
          min="0"
          max="100"
          step="0.1"
          inputMode="decimal"
          value={
            isFocused ? parseFloat(localValue).toFixed(2) : weight.toFixed(2)
          }
          onFocus={() => {
            setIsFocused(true);
            setLocalValue(weight.toString());
          }}
          onChange={(e) => setLocalValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              commit();
              e.currentTarget.blur();
            }
          }}
          onBlur={() => {
            commit();
            setIsFocused(false);
          }}
          className="w-25 rounded-sm border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 pl-2 pr-6 py-0.5 text-right text-sm outline-hidden focus:ring-1 focus:ring-blue-500 appearance-none"
        />
        <span className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-gray-500 pointer-events-none">
          %
        </span>
      </div>
      <span>
        {amount.toFixed(2)} {symbol ? `${symbol}` : ''}
      </span>
    </div>
  );
};
