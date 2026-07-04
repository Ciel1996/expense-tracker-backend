import type { GetServerSideProps, NextPage } from "next";
import {
  useGetPotExpenses,
  useGetUsers,
  useDeletePot,
  useGetPots, useArchive, useUnarchive, getGetPotsQueryKey, useCurrentUser, usePayPot, getGetPotExpensesQueryKey
} from "@./expense-tracker-client";
import React, { useState } from "react";
import { useRouter } from "next/router";
import { ExpensesTable } from "../../components/expenses/expenses-table";
import { NewExpenseModal } from "../../components/expenses/new-expense-modal";
import { useQueryClient } from "@tanstack/react-query";

type Props = { id: number };

const PotDetails: NextPage<Props> = ({ id }) => {
  const router = useRouter();
  const { data: user } = useCurrentUser();
  const [isNewExpenseOpen, setNewExpenseOpen] = useState(false);
  const { data: expenses, isLoading, isError } = useGetPotExpenses(id);
  const { data: users, isLoading: isLoadingUsers, isError: isErrorUsers } = useGetUsers();
  const { data: pots, isLoading: isLoadingPots, isError: isErrorPots } = useGetPots();
  const pot = (pots ?? []).find((p) => p.id === id) ?? null;
  const pot_name = pot?.name ?? `Pot ${id}`;
  const isArchived = pot?.archived ?? false;

  const queryClient = useQueryClient();

  const { mutate: archivePot, isPending: isArchiving } = useArchive({
    mutation: {
      onSuccess: async () => {
        await queryClient.invalidateQueries({ queryKey: getGetPotsQueryKey() });
      },
    },
  });

  const { mutate: unarchivePot, isPending: isUnarchiving } = useUnarchive({
    mutation: {
      onSuccess: async () => {
        await queryClient.invalidateQueries({ queryKey: getGetPotsQueryKey() });
      },
    },
  });

  const { mutate: deletePot, isPending: isDeleting, error: deleteError } = useDeletePot({
    mutation: {
      onSuccess: async () => {
        // Navigate back to overview
        await router.push("/");
      },
    },
  });

  const { mutate: payPot, isPending: isPaying, error: payError } = usePayPot({
    mutation: {
      onSuccess: async () => {
        await queryClient.invalidateQueries({ queryKey: getGetPotsQueryKey() });
        await queryClient.invalidateQueries({ queryKey: getGetPotExpensesQueryKey(id) });
      },
    },
  });

  if (isLoading || isLoadingUsers || isLoadingPots) {
    return <div className="p-4 text-sm text-gray-500">Loading expenses…</div>;
  }

  if (isError || isErrorUsers || isErrorPots) {
    return (
      <div className="p-4 text-sm text-red-600">Failed to load expenses. Please try again.</div>
    );
  }

  // Compute derived state
  const hasExpenses = expenses && expenses.length > 0;
  const allExpensesPaid = !hasExpenses || (expenses && expenses.every(e => (e.splits ?? []).every(s => s.is_paid)));
  const isOwner = pot?.owner_id === user?.uuid;
  const canArchive = allExpensesPaid;
  const canDelete = allExpensesPaid && !isArchived;
  const canPay = hasExpenses && !allExpensesPaid;

  const handlePayment = () => {
    if (!canPay || isPaying) return;
    const ok = window.confirm("Are you sure you want to pay this pot? This action cannot be undone.");
    if (!ok) return;
    payPot({ potId: id });
  };

  const handleDelete = () => {
    if (!canDelete || isDeleting) return;
    const ok = window.confirm("Are you sure you want to delete this pot? This action cannot be undone.");
    if (!ok) return;
    deletePot({ potId: id });
  };

  const handleArchive = () => {
    if (!canArchive || isArchiving || isArchived) return;
    archivePot({ potId: id });
  };

  const handleUnarchive = () => {
    if (isUnarchiving || !isArchived) return;
    unarchivePot({ potId: id });
  };

  const actionSection = (
    <div className="fixed bottom-6 left-6 flex gap-4 z-10">
      {(isOwner && !isArchived && (canPay || isPaying)) && (
        <button
          onClick={handlePayment}
          disabled={isPaying}
          className="w-14 h-14 rounded-full bg-green-600 text-white shadow-xl flex items-center justify-center text-2xl z-10 transition-transform active:scale-95 disabled:opacity-50"
          title={isPaying ? "Paying…" : "Mark as Paid"}
        >
          {isPaying ? "..." : "✅"}
        </button>
      )}

      {(isOwner && !isArchived && (canDelete || isDeleting)) && (
        <button
          onClick={handleDelete}
          disabled={isDeleting}
          className="w-14 h-14 rounded-full bg-red-600 text-white shadow-xl flex items-center justify-center text-2xl z-10 transition-transform active:scale-95 disabled:opacity-50"
          title={isDeleting ? "Deleting…" : "Delete Pot"}
        >
          {isDeleting ? "..." : "🗑️"}
        </button>
      )}

      {(isOwner && !isArchived && (canArchive || isArchiving)) && (
        <button
          onClick={handleArchive}
          disabled={isArchiving}
          className="w-14 h-14 rounded-full bg-amber-600 text-white shadow-xl flex items-center justify-center text-2xl z-10 transition-transform active:scale-95 disabled:opacity-50"
          title={isArchiving ? "Archiving..." : "Archive Pot"}
        >
          {isArchiving ? "..." : "📦"}
        </button>
      )}

      {isOwner && isArchived && (
        <button
          onClick={handleUnarchive}
          disabled={isUnarchiving}
          className="w-14 h-14 rounded-full bg-amber-600 text-white shadow-xl flex items-center justify-center text-2xl z-10 transition-transform active:scale-95 disabled:opacity-50"
          title={isUnarchiving ? "Unarchiving..." : "Unarchive Pot"}
        >
          {isUnarchiving ? "..." : "📤"}
        </button>
      )}
    </div>
  );

  if (!users || users.length === 0) {
    return <div className="p-4 text-sm text-gray-500">Could not connect to user database</div>;
  }

  return (
    <div className="p-4">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <h1 className="text-xl font-semibold">{pot_name}</h1>
          {isArchived && (
            <span className="bg-gray-100 text-gray-800 text-xs font-medium px-2.5 py-0.5 rounded-sm dark:bg-gray-700 dark:text-gray-300">
              Archived
            </span>
          )}
        </div>
      </div>

      {/* Content */}
      {hasExpenses ? (
        <div className="mt-4">
          <ExpensesTable expenses={expenses} users={users} potId={id} isArchived={isArchived} />
        </div>
      ) : (
        <div className="mt-4 text-sm text-gray-500">Nothing to display</div>
      )}

      {/* Show actions if allowed */}
      {actionSection}

      {!isArchived && (
        <button
          onClick={() => setNewExpenseOpen(true)}
          disabled={!pot}
          className="fixed bottom-6 right-6 w-14 h-14 rounded-full bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 shadow-xl flex items-center justify-center text-3xl z-10 transition-transform active:scale-95"
          title={!pot ? "Loading pot…" : "Add a new expense"}
        >
          +
        </button>
      )}

      {/* Modal */}
      <NewExpenseModal open={isNewExpenseOpen} onClose={() => setNewExpenseOpen(false)} pot={pot} potId={id} />
    </div>
  );
};

export const getServerSideProps: GetServerSideProps<Props> = async (ctx) => {
  const rawId = ctx.params?.id;

  // Validate and parse id
  if (Array.isArray(rawId)) {
    return { notFound: true };
  }

  const idNum = Number(rawId);

  if (!Number.isFinite(idNum)) {
    return { notFound: true };
  }

  return {
    props: { id: idNum },
  };
};

export default PotDetails;
