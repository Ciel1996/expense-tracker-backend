import type { GetServerSideProps, NextPage } from "next";
import {useGetPotExpenses, useGetUsers} from "@./expense-tracker-client";

type Props = { id: number };

const PotDetails: NextPage<Props> = ({ id }) => {
  const { data: expenses, isLoading, isError } = useGetPotExpenses(id);
  const { data: users, isLoading: isLoadingUsers, isError: isErrorUsers} = useGetUsers();

  if (isLoading || isLoadingUsers) {
    return <div className="p-4 text-sm text-gray-500">Loading expenses…</div>;
  }

  if (isError || isErrorUsers) {
    return (
      <div className="p-4 text-sm text-red-600">Failed to load expenses. Please try again.</div>
    );
  }

  if (!expenses || expenses.length === 0) {
    return <div className="p-4 text-sm text-gray-500">Nothing to display</div>;
  }

  if (!users || users.length === 0) {
    return <div className="p-4 text-sm text-gray-500">Could not connect to user database</div>;
  }

  return (
    <div className="p-4">
      <h1 className="mb-4 text-xl font-semibold">Pot {id} — Expenses</h1>

      <div className="overflow-x-auto rounded border border-gray-200 dark:border-gray-800">
        <table className="min-w-full table-auto text-left">
          <thead className="bg-gray-50 text-gray-600 dark:bg-gray-800 dark:text-gray-300">
            <tr>
              <th className="px-4 py-2">Description</th>
              <th className="px-4 py-2">Amount</th>
              <th className="px-4 py-2">Owner</th>
              <th className="px-4 py-2">Paid</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100 dark:divide-gray-800">
            {expenses.map((e) => {
              const paid = e.splits?.filter((s) => s.is_paid).length ?? 0;
              const total = e.splits?.length ?? 0;
              const currencySymbol = e.currency?.symbol ?? "";
              const amount = `${currencySymbol}${e.sum?.toFixed?.(2) ?? e.sum}`;

              return (
                <tr key={e.id} className="hover:bg-gray-50 dark:hover:bg-gray-800">
                  <td className="px-4 py-2">{e.description}</td>
                  <td className="px-4 py-2 whitespace-nowrap">{amount}</td>
                  <td className="px-4 py-2 text-sm text-gray-600 dark:text-gray-300">
                    {users.find(u => u.uuid == e.owner_id)?.name}
                  </td>
                  <td className="px-4 py-2 text-sm text-gray-600 dark:text-gray-300">{paid}/{total}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
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
