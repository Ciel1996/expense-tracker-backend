"use client";

import {FC, useMemo, useState} from "react";
import {
  useAddUserToPot,
  useCreatePot,
  useGetCurrencies,
  useGetUsers,
  getGetPotsQueryKey,
  useCurrentUser
} from "@./expense-tracker-client";
import {useQueryClient} from "@tanstack/react-query";

export const NewPotModal: FC<{ open: boolean; onClose: () => void }> = ({ open, onClose }) => {
  const [name, setName] = useState("");
  const [currencyId, setCurrencyId] = useState<number | "">("");
  const [selectedUserIds, setSelectedUserIds] = useState<string[]>([]);

  const queryClient = useQueryClient();

  const { data: currencies } = useGetCurrencies();
  const { data: currentUser } = useCurrentUser();
  const { data: users } = useGetUsers();
  const filteredUsers = users?.filter(u => u.uuid != currentUser?.uuid);

  const createPot = useCreatePot();
  const addUserToPot = useAddUserToPot();

  const canSubmit = useMemo(
    () => name.trim().length > 0 && typeof currencyId === "number", [name, currencyId]);

  const resetForm = () => {
    setName("");
    setCurrencyId("");
    setSelectedUserIds([]);
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  const onToggleUser = (uuid: string) => {
    setSelectedUserIds(prev => prev.includes(uuid) ? prev.filter(id => id !== uuid) : [...prev, uuid]);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!canSubmit || typeof currencyId !== "number") return;

    try {
      const pot = await createPot.mutateAsync({ data: { name: name.trim(), default_currency_id: currencyId } });
      // Add selected users to pot sequentially
      for (const uuid of selectedUserIds) {
        await addUserToPot.mutateAsync({ potId: pot.id, data: { user_id: uuid } });
      }
      // invalidate pots list
      await queryClient.invalidateQueries({ queryKey: getGetPotsQueryKey() });
      handleClose();
    } catch (err) {
      // Could show a toast later
      console.error("Failed to create pot:", err);
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50">
      <div className="absolute inset-0 bg-black/30" onClick={handleClose} />
      <div className="absolute inset-0 overflow-y-auto">
        <div className="flex min-h-full items-center justify-center p-4">
          <div className="w-full max-w-lg overflow-hidden rounded-lg bg-white dark:bg-gray-900 p-6 text-left shadow-xl">
            <div className="text-lg font-medium text-gray-900 dark:text-gray-100">
              Create New Pot
            </div>

            <form className="mt-4 space-y-4" onSubmit={handleSubmit}>
              <div>
                <label className="block text-sm font-medium mb-1">Name</label>
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
                  placeholder="Ebeneezer's Birthday"
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Currency</label>
                <select
                  value={currencyId}
                  onChange={(e) => setCurrencyId(e.target.value ? Number(e.target.value) : "")}
                  className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
                  required
                >
                  <option value="" disabled>Select a currency</option>
                  {(currencies ?? []).map(c => (
                    <option key={c.id} value={c.id}>{c.symbol} — {c.name}</option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">Add users</label>
                <div className="max-h-40 overflow-auto rounded-md border border-gray-200 dark:border-gray-700 divide-y divide-gray-200 dark:divide-gray-800">
                  {(filteredUsers ?? []).map(u => (
                    <label key={u.uuid} className="flex items-center gap-3 px-3 py-2 text-sm">
                      <input
                        type="checkbox"
                        checked={selectedUserIds.includes(u.uuid)}
                        onChange={() => onToggleUser(u.uuid)}
                        className="h-4 w-4"
                      />
                      <span className="truncate">{u.name} <span className="text-xs text-gray-500">({u.uuid})</span></span>
                    </label>
                  ))}
                  {(!users || users.length === 0) && (
                    <div className="px-3 py-2 text-sm text-gray-500">No users available.</div>
                  )}
                </div>
              </div>

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
                  disabled={!canSubmit || createPot.isPending || addUserToPot.isPending}
                  className="px-4 py-2 rounded-md bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
                >
                  {createPot.isPending ? "Creating..." : "Create"}
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
};
