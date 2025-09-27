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
  const [payerId, setPayerId] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  const addExpense = useAddExpense();
  const queryClient = useQueryClient();

  const participants = pot?.users ?? [];
  const currency = pot?.default_currency;

  const canSubmit = useMemo(() => {
    const amount = Number(amountInput);
    return (
      !!pot && description.trim().length > 0 && !Number.isNaN(amount) && amount > 0 && payerId.length > 0
    );
  }, [pot, description, amountInput, payerId]);

  const resetForm = () => {
    setDescription("");
    setAmountInput("");
    setPayerId("");
    setError(null);
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!pot || !canSubmit || !currency) return;

    try {
      setError(null);
      // Work in cents to avoid floating point issues
      const totalCents = Math.round(Number(amountInput) * 100);
      const n = participants.length;
      const baseShare = Math.floor(totalCents / n);
      const remainder = totalCents - baseShare * n;

      const ownerId = pot.owner_id;

      const splits = participants.map((u) => {
        const extra = u.uuid === ownerId ? remainder : 0;
        const cents = baseShare + extra;
        return {
          user_id: u.uuid,
          amount: cents / 100,
          is_paid: u.uuid === payerId,
        };
      });

      await addExpense.mutateAsync({
        potId: potId,
        data: {
          currency_id: currency.id,
          description: description.trim(),
          splits,
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
                <label className="block text-sm font-medium mb-1">Payer</label>
                <select
                  value={payerId}
                  onChange={(e) => setPayerId(e.target.value)}
                  className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
                  required
                >
                  <option value="" disabled>
                    Select the payer
                  </option>
                  {participants.map((u) => (
                    <option key={u.uuid} value={u.uuid}>
                      {u.name}
                    </option>
                  ))}
                </select>
                <p className="mt-1 text-xs text-gray-500">The payer's share will be marked as already paid.</p>
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
