"use client";

import React, { FC, useEffect, useState } from "react";

export interface AbsoluteSplitInputProps {
  amount: number;
  totalAmount: number;
  symbol?: string;
  onCommitWeight?: (newWeight: number) => void;
  onCommitAmount?: (newAmount: number) => void;
}

export const AbsoluteSplitInput: FC<AbsoluteSplitInputProps> = ({
  amount,
  totalAmount,
  symbol,
  onCommitWeight,
  onCommitAmount,
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const [localValue, setLocalValue] = useState(amount.toFixed(2));

  useEffect(() => {
    if (!isFocused) {
      setLocalValue(amount.toFixed(2));
    }
  }, [amount, isFocused]);

  const commit = () => {
    const parsed = parseFloat(localValue);
    if (!isNaN(parsed) && totalAmount > 0) {
      const clampedAmount = Math.max(0, parsed);
      if (onCommitAmount) {
        onCommitAmount(clampedAmount);
      } else if (onCommitWeight) {
        const newWeight = Math.min(100, (clampedAmount / totalAmount) * 100);
        onCommitWeight(newWeight);
      }
    } else {
      setLocalValue(amount.toFixed(2));
    }
  };

  return (
    <div className="relative">
      <input
        type="number"
        step="0.01"
        inputMode="decimal"
        value={isFocused ? localValue : amount.toFixed(2)}
        onFocus={() => {
          setIsFocused(true);
          setLocalValue(amount.toFixed(2));
        }}
        onChange={(e) => setLocalValue(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
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
        {symbol}
      </span>
    </div>
  );
};
