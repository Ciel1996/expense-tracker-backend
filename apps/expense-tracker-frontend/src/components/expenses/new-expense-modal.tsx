"use client";

import React, { FC, useMemo, useState } from "react";
import {
  PotDTO,
  useAddExpense,
  getGetPotExpensesQueryKey,
} from "@./expense-tracker-client";
import { useQueryClient } from "@tanstack/react-query";

interface NewExpenseModalProps {
  open: boolean;
  onClose: () => void;
  pot: PotDTO | null;
  potId: number;
}

export const NewExpenseModal: FC<NewExpenseModalProps> = ({ open, onClose, pot, potId }) => {
  const [description, setDescription] = useState("");
  const [amountInput, setAmountInput] = useState("");
  const [weights, setWeights] = useState<Record<string, number>>({});
  const [error, setError] = useState<string | null>(null);

  const addExpense = useAddExpense();
  const queryClient = useQueryClient();

  const participants = pot?.users ?? [];
  const currency = pot?.default_currency;

  // Initialize weights when participants change
  React.useEffect(() => {
    if (participants.length > 0) {
      const initialWeights: Record<string, number> = {};
      const share = 100 / participants.length;
      participants.forEach((u) => {
        initialWeights[u.uuid] = Number(share.toFixed(1));
      });
      setWeights(initialWeights);
    }
  }, [participants]);

  const totalWeight = useMemo(() => {
    return Object.values(weights).reduce((sum, w) => sum + w, 0);
  }, [weights]);

  const canSubmit = useMemo(() => {
    const amount = Number(amountInput);
    return (
      !!pot &&
      description.trim().length > 0 &&
      !Number.isNaN(amount) &&
      amount > 0 &&
      totalWeight > 0
    );
  }, [pot, description, amountInput, totalWeight]);

  const resetForm = () => {
    setDescription("");
    setAmountInput("");
    setError(null);
    if (participants.length > 0) {
      const initialWeights: Record<string, number> = {};
      const share = 100 / participants.length;
      participants.forEach((u) => {
        initialWeights[u.uuid] = Number(share.toFixed(1));
      });
      setWeights(initialWeights);
    }
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  const handleWeightChange = (userId: string, newWeight: number) => {
    setWeights((prev) => {
      const otherUserIds = Object.keys(prev).filter((id) => id !== userId);
      if (otherUserIds.length === 0) {
        return { [userId]: 100 };
      }

      // Clamp newWeight between 0 and 100
      const clampedNewWeight = Math.min(100, Math.max(0, newWeight));
      const remainingTotal = 100 - clampedNewWeight;
      const currentOtherTotal = otherUserIds.reduce((sum, id) => sum + prev[id], 0);

      const nextWeights = { ...prev, [userId]: clampedNewWeight };

      if (currentOtherTotal > 0) {
        // Distribute remaining total proportionally
        otherUserIds.forEach((id) => {
          nextWeights[id] = Number(((prev[id] / currentOtherTotal) * remainingTotal).toFixed(1));
        });
      } else {
        // If others were all zero, distribute equally
        const equalShare = remainingTotal / otherUserIds.length;
        otherUserIds.forEach((id) => {
          nextWeights[id] = Number(equalShare.toFixed(1));
        });
      }

      // Fix rounding errors to ensure it sums to 100
      const newTotal = Object.values(nextWeights).reduce((s, w) => s + w, 0);
      const diff = 100 - newTotal;
      if (Math.abs(diff) > 0.01 && otherUserIds.length > 0) {
        // Add difference to the first other user to maintain sum
        nextWeights[otherUserIds[0]] = Number((nextWeights[otherUserIds[0]] + diff).toFixed(1));
      }

      return nextWeights;
    });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!pot || !canSubmit || !currency) return;

    try {
      setError(null);
      // Work in cents to avoid floating point issues
      const totalCents = Math.round(Number(amountInput) * 100);

      const splits = participants.map((u) => {
        const weight = weights[u.uuid] ?? 0;
        const shareRatio = totalWeight > 0 ? weight / totalWeight : 0;
        // Basic calculation, but we need to handle rounding so it sums up exactly to totalCents
        return {
          user_id: u.uuid,
          cents: Math.floor(totalCents * shareRatio),
          ratio: shareRatio
        };
      });

      // Distribute remaining cents due to flooring
      let distributedCents = splits.reduce((sum, s) => sum + s.cents, 0);
      let remainingCents = totalCents - distributedCents;

      // Give remainder to those with highest ratio/weight to minimize impact
      const sortedByWeight = [...splits].sort((a, b) => b.ratio - a.ratio);
      for (let i = 0; i < remainingCents; i++) {
        sortedByWeight[i].cents += 1;
      }

      const finalSplits = splits.map(s => ({
        user_id: s.user_id,
        amount: s.cents / 100
      }));

      await addExpense.mutateAsync({
        potId: potId,
        data: {
          currency_id: currency.id,
          description: description.trim(),
          splits: finalSplits,
        },
      });

      // Refresh pot expenses list
      await queryClient.invalidateQueries({ queryKey: getGetPotExpensesQueryKey(potId) });
      handleClose();
    } catch (err) {
      console.error("Failed to add expense", err);
      setError("Failed to add expense. Please try again.");
    }
  };

  if (!open || !pot) return null;

  return (
    <div className="fixed inset-0 z-50">
      <div className="absolute inset-0 bg-black/30" onClick={handleClose} />
      <div className="absolute inset-0 overflow-y-auto">
        <div className="flex min-h-full items-center justify-center p-4">
          <div className="w-full max-w-md overflow-hidden rounded-lg bg-white dark:bg-gray-900 p-6 text-left shadow-xl">
            <div className="text-lg font-medium text-gray-900 dark:text-gray-100">Add Expense</div>

            <form className="mt-4 space-y-4" onSubmit={handleSubmit}>
              <div>
                <label className="block text-sm font-medium mb-1">Description</label>
                <input
                  type="text"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="Dinner, tickets, etc."
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Amount ({currency?.symbol})</label>
                <input
                  type="number"
                  inputMode="decimal"
                  min="0"
                  step="0.01"
                  value={amountInput}
                  onChange={(e) => setAmountInput(e.target.value)}
                  className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="0.00"
                  required
                />
              </div>

              <div>
                <div className="flex items-center justify-between mb-1">
                  <label className="block text-sm font-medium">Split Ratios</label>
                  <button
                    type="button"
                    onClick={() => {
                      const reset: Record<string, number> = {};
                      const share = 100 / participants.length;
                      participants.forEach(u => reset[u.uuid] = Number(share.toFixed(1)));
                      setWeights(reset);
                    }}
                    className="text-xs text-blue-600 hover:text-blue-700 font-medium"
                  >
                    Reset to equal
                  </button>
                </div>
                <div className="space-y-3 rounded-md border border-gray-200 dark:border-gray-800 p-3 bg-gray-50/50 dark:bg-gray-800/50">
                  {participants.map((u) => {
                    const weight = weights[u.uuid] ?? 0;
                    const percentage = totalWeight > 0 ? (weight / totalWeight) * 100 : 0;
                    const amount = totalWeight > 0 ? (Number(amountInput) * weight) / totalWeight : 0;

                    return (
                      <div key={u.uuid} className="flex flex-col gap-1">
                        <div className="flex justify-between text-xs font-medium">
                          <span>{u.name}</span>
                          <span className="text-gray-500">
                            {percentage.toFixed(1)}% ({currency?.symbol}{amount.toFixed(2)})
                          </span>
                        </div>
                        <div className="flex items-center gap-3">
                          <input
                            type="range"
                            min="0"
                            max="100"
                            step="1"
                            value={weight}
                            onChange={(e) => {
                              handleWeightChange(u.uuid, parseFloat(e.target.value));
                            }}
                            className="flex-1 h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-600"
                          />
                          <div className="relative">
                            <input
                              type="number"
                              min="0"
                              max="100"
                              step="0.1"
                              value={weight}
                              onChange={(e) => {
                                const val = parseFloat(e.target.value);
                                handleWeightChange(u.uuid, isNaN(val) ? 0 : val);
                              }}
                              className="w-20 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 pl-2 pr-6 py-0.5 text-right text-sm outline-none focus:ring-1 focus:ring-blue-500 appearance-none"
                            />
                            <span className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-gray-500 pointer-events-none">%</span>
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
                {totalWeight === 0 && (
                  <p className="mt-1 text-xs text-red-500">At least one person must have a share &gt; 0</p>
                )}
              </div>

              {error && (
                <div className="text-sm text-red-600" role="alert">
                  {error}
                </div>
              )}

              <div className="flex justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={handleClose}
                  className="px-4 py-2 rounded-md border border-gray-300 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={!canSubmit || addExpense.isPending}
                  className="px-4 py-2 rounded-md bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
                >
                  {addExpense.isPending ? "Adding…" : "Add Expense"}
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
};
