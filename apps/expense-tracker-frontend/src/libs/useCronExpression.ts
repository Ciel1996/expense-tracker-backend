'use client';

import { useState, useCallback, useEffect } from 'react';

const getRecurrenceFromCron = (cron: string): string => {
  console.log('cron', cron);
  const parts = cron.split(' ');
  if (parts.length !== 6) {
    return 'monthly';
  }
  const [, , , , month, dayOfWeek] = parts;

  if (dayOfWeek !== '*') {
    return 'weekly';
  }

  if (month === '*') {
      return 'monthly';
  }

  return 'yearly';
};

const getDateTimeFromCron = (cron: string): string => {
  if (cron.trim() === '') return '';
  const recurrence = getRecurrenceFromCron(cron);

  const parts = cron.split(' ');
  const [, minute, hour, day, month, dayOfWeek] = parts;
  const date = new Date();

  if (recurrence == "yearly") {
    date.setMonth(parseInt(month) - 1);
    date.setDate(parseInt(day));
    date.setHours(parseInt(hour));
    date.setMinutes(parseInt(minute));
  }

  if (recurrence == "monthly") {
    date.setDate(parseInt(day));
    date.setHours(parseInt(hour));
    date.setMinutes(parseInt(minute));
  }

  if (recurrence == "weekly") {
    date.setDate(date.getDate() + (parseInt(dayOfWeek) - date.getDay()));
    date.setHours(parseInt(hour));
    date.setMinutes(parseInt(minute));
  }

  return date.toISOString().slice(0, 16);
};

export const useCronExpression = (
  initialRecurrence?: string,
  initialDateTime?: string,
  initialCronExpression?: string,
) => {
  const [recurrence, setRecurrence] = useState(initialRecurrence || '');
  const [dateTime, setDateTime] = useState(initialDateTime || '');
  const [cronExpression, setCronExpression] = useState(initialCronExpression || '');

  useEffect(() => {
    setCronExpression(initialCronExpression || '');
    setRecurrence(
      initialRecurrence || getRecurrenceFromCron(initialCronExpression || '')
    );
    setDateTime(
      initialDateTime || getDateTimeFromCron(initialCronExpression || '')
    )
  }, [initialCronExpression, initialDateTime]);

  const onDateTimeChange = useCallback(
    (dateT: string) => {
      setDateTime(dateT);
      if (!dateT) {
        setCronExpression('');
        return;
      }
      calculateCronExpression(dateT, recurrence);
    },
    [dateTime, recurrence]
  );

  const calculateCronExpression =
    (val: string, recurrence: string) => {
    const date = new Date(val);
    const minute = date.getMinutes();
    const hour = date.getHours();
    const dayOfMonth = date.getDate();
    const month = date.getMonth() + 1; // getMonth is 0-indexed
    const dayOfWeek = date.getDay(); // Sunday is 0, Monday is 1...

    let cron = '';
    switch (recurrence) {
      case 'weekly':
        cron = `0 ${minute} ${hour} * * ${dayOfWeek}`;
        break;
      case 'monthly':
        cron = `0 ${minute} ${hour} ${dayOfMonth} * *`;
        break;
      case 'yearly':
        cron = `0 ${minute} ${hour} ${dayOfMonth} ${month} *`;
        break;
      default:
        cron = `0 ${minute} ${hour} ${dayOfMonth} * *`;
    }
    setCronExpression(cron);
  };

  const onRecurrenceChange = useCallback(
    (rec: string) => {
      setRecurrence(rec);
      calculateCronExpression(dateTime, rec);
    },
    [dateTime, recurrence]
  );

  return {
    recurrence,
    setRecurrence,
    dateTime,
    setDateTime,
    cronExpression,
    setCronExpression,
    onDateTimeChange,
    onRecurrenceChange,
  };
};
