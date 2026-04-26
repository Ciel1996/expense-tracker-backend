import type { GetServerSideProps, NextPage } from 'next';
import React, { useEffect, useMemo, useState } from 'react';
import {
  useAddUsersToTemplate,
  useCurrentUser,
  useDeletePotTemplate,
  useGetCurrencies,
  useGetPotTemplateById,
  useGetUsers,
  UserDTO,
  useRemoveUsersFromTemplate,
  useUpdateTemplate,
} from '@./expense-tracker-client';
import { router } from 'next/client';
import { useCronExpression } from '../../libs/useCronExpression';

type Props = { id: number };

const TemplateDetails: NextPage<Props> = ({ id }) => {
  const { data: currentUser } = useCurrentUser();
  const {
    data: template,
    isLoading: isLoadingPot,
    isError,
  } = useGetPotTemplateById(id);
  const {
    data: users,
    isLoading: isLoadingUsers,
    isError: isErrorUsers,
  } = useGetUsers();
  const { data: currencies } = useGetCurrencies();
  const [name, setName] = useState(template?.name ?? `Template ${id}`);
  const [currencyId, setCurrencyId] = useState<number>(
    template?.default_currency.id ?? 1
  );
  const [templateUsers, setTemplateUsers] = useState<UserDTO[]>(
    template?.users ?? []
  );
  const [selectedUserIds, setSelectedUserIds] = useState<string[]>(
    templateUsers.map((user) => user.uuid) ?? []
  );

  // backend works with cron syntax
  const {
    recurrence,
    dateTime,
    cronExpression,
    onRecurrenceChange,
    onDateTimeChange,
  } = useCronExpression(
    undefined,
    undefined,
    template?.cron_expression);

  const {
    mutate: removeUsers,
    isPending: isRemovingUsers,
    error: removeUsersError,
  } = useRemoveUsersFromTemplate({
    mutation: {
      onSuccess: async () => {
        await router.push(`/?tab=templates`);
      },
    },
  });

  const {
    mutate: addUsers,
    isPending: isAddingUsers,
    error: addUsersError,
  } = useAddUsersToTemplate({
    mutation: {
      onSuccess: async () => {
        await router.push(`/?tab=templates`);
      },
    },
  });

  const {
    mutate: deleteTemplate,
    isPending: isDeletingTemplate,
    error: deleteTemplateError,
  } = useDeletePotTemplate({
    mutation: {
      onSuccess: async () => {
        await router.push(`/?tab=templates`);
      },
    },
  });

  const {
    mutate: updateTemplate,
    isPending: isUpdatingTemplate,
    error: updateTemplateError,
  } = useUpdateTemplate({
    mutation: {
      onSuccess: async () => {
        await router.push(`/?tab=templates`);
      },
    },
  });

  useEffect(() => {
    if (template) {
      setName(template.name);
      setCurrencyId(template.default_currency.id);
      setTemplateUsers(template.users);
      setSelectedUserIds(template.users.map((user) => user.uuid));

    }
  }, [template]);

  const canSubmit = useMemo(
    () =>
      name.trim().length > 0 &&
      cronExpression.trim().length > 0 &&
      recurrence !== '',
    [name, currencyId, cronExpression, recurrence, selectedUserIds]
  );

  const filteredUsers =
    users?.filter((user) => user.uuid != template?.owner.uuid) ?? [];

  const handleSubmit = (e: React.FormEvent) => {
    // required so that the page is not immediately refreshed
    e.preventDefault();
    if (!canSubmit) return;

    submitRemoveUsers();
    submitAddUsers();
    submitUpdate();
  };

  const submitUpdate = () => {
    // TODO: check if update has to be called!
    // TODO: update cron expression
    updateTemplate({
      templateId: id,
      data: {
        name: name.trim(),
        default_currency_id: currencyId,
        cron_expression: cronExpression,
      },
    });
  };

  const submitRemoveUsers = () => {
    // users that have been deselected
    let usersToRemove = templateUsers.filter(
      (user) => !selectedUserIds.includes(user.uuid)
    );

    if (usersToRemove.length === 0) return;

    removeUsers({
      templateId: id,
      data: {
        users: usersToRemove.map((user) => user.uuid),
      },
    });
  };

  const submitAddUsers = () => {
    // users that have been selected
    let usersToAdd = filteredUsers.filter((user) => {
      if (templateUsers.includes(user)) return false;
      return selectedUserIds.includes(user.uuid);
    });

    if (usersToAdd.length === 0) return;

    addUsers({
      templateId: id,
      data: { users: usersToAdd.map((user) => user.uuid) },
    });
  };

  const handleDelete = () => {
    if (window.confirm('Are you sure you want to delete this template?')) {
      deleteTemplate({ templateId: id });
    }
  };

  const handleClose = async () => {
    await router.push(`/?tab=templates`);
  };

  const onToggleUser = (uuid: string) => {
    setSelectedUserIds((prev) =>
      prev.includes(uuid) ? prev.filter((id) => id !== uuid) : [...prev, uuid]
    );
  };

  if (isLoadingPot || isLoadingUsers) {
    return (
      <div className="p-4 text-sm text-gray-500">
        Loading template details...
      </div>
    );
  }

  if (isError || isErrorUsers) {
    if (!template) {
      return (
        <div className="p-4 text-sm text-gray-500">Template not found</div>
      );
    }

    return (
      <div className="p-4 text-sm text-red-600">
        Error loading template details
      </div>
    );
  }

  if (!users || users.length === 0) {
    return (
      <div className="p-4 text-sm text-gray-500">
        Could not connect to user database
      </div>
    );
  }

  return (
    <form className="mt-4 space-y-4" onSubmit={handleSubmit}>
      <div>
        <label className="block text-sm font-medium mb-1">Name</label>
        <input
          type="text"
          maxLength={24}
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
          placeholder="Home {year}/{month}"
          required
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-1">Currency</label>
        <select
          value={currencyId}
          onChange={(e) =>
            setCurrencyId(e.target.value ? Number(e.target.value) : 1)
          }
          className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
          required
        >
          <option value="" disabled>
            Select a currency
          </option>
          {(currencies ?? []).map((c) => (
            <option key={c.id} value={c.id}>
              {c.symbol} — {c.name}
            </option>
          ))}
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium mb-2">Add users</label>
        <div className="max-h-40 overflow-auto rounded-md border border-gray-200 dark:border-gray-700 divide-y divide-gray-200 dark:divide-gray-800">
          {(filteredUsers ?? []).map((u) => (
            <label
              key={u.uuid}
              className="flex items-center gap-3 px-3 py-2 text-sm"
            >
              <input
                type="checkbox"
                checked={selectedUserIds.includes(u.uuid)}
                onChange={() => onToggleUser(u.uuid)}
                className="h-4 w-4"
              />
              <span className="truncate">{u.name}</span>
            </label>
          ))}
          {(!users || users.length === 0) && (
            <div className="px-3 py-2 text-sm text-gray-500">
              No users available.
            </div>
          )}
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium mb-1">Recurrence</label>
          <select
            value={recurrence}
            onChange={(e) => onRecurrenceChange(e.target.value)}
            className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
            required
          >
            <option value="weekly">Weekly</option>
            <option value="monthly">Monthly</option>
            <option value="yearly">Yearly</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">
            Schedule (Day & Time)
          </label>
          <input
            type="datetime-local"
            value={dateTime}
            onChange={(e) => onDateTimeChange(e.target.value)}
            className="w-full rounded-md border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 px-3 py-2 outline-none focus:ring-2 focus:ring-blue-500"
            required
          />
        </div>
      </div>

      {cronExpression && (
        <div className="text-xs text-gray-500 dark:text-gray-400">
          Resulting Cron:{' '}
          <code className="bg-gray-100 dark:bg-gray-800 px-1 rounded">
            {cronExpression}
          </code>
        </div>
      )}

      <div className="flex justify-end gap-2 pt-2">
        {currentUser?.uuid === template?.owner.uuid && (
          <button
            type="button"
            onClick={handleDelete}
            className="px-4 py-2 rounded-md text-white bg-red-600 hover:bg-red-700"
          >
            Delete
          </button>
        )}

        <button
          type="button"
          onClick={handleClose}
          className="px-4 py-2 rounded-md border border-gray-300 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800"
        >
          Cancel
        </button>
        <button
          type="submit"
          disabled={!canSubmit || isRemovingUsers}
          className="px-4 py-2 rounded-md bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {isRemovingUsers ? 'Updating...' : 'Update'}
        </button>
      </div>
    </form>
  );
}

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

export default TemplateDetails;
