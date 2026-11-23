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
    <div className="flex w-25 items-center rounded-sm border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 focus-within:ring-1 focus-within:ring-blue-500">
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
        className="min-w-0 flex-1 bg-transparent pl-2 py-0.5 text-right text-base pointer-fine:text-sm outline-hidden appearance-none"
      />
      {symbol && (
        <span className="shrink-0 pl-1.5 pr-2 text-xs text-gray-500 pointer-events-none select-none">
          {symbol}
        </span>
      )}
    </div>
  );
};
