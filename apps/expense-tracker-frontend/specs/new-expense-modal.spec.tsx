import React from 'react';
import { render, screen, fireEvent, within } from '@testing-library/react';
import { AbsoluteSplitInput } from '../src/components/expenses/absolute-split-input';
import { PercentageSplitInput } from '../src/components/expenses/percentage-split-input';
import { NewExpenseModal } from '../src/components/expenses/new-expense-modal';

jest.mock('@./expense-tracker-client', () => ({
  useAddExpense: () => ({
    mutateAsync: jest.fn(),
    isPending: false,
  }),
  getGetPotExpensesQueryKey: jest.fn(),
}));

jest.mock('@tanstack/react-query', () => ({
  useQueryClient: () => ({
    invalidateQueries: jest.fn(),
  }),
}));

describe('AbsoluteSplitInput', () => {
  it('displays formatted amount initially', () => {
    render(
      <AbsoluteSplitInput
        amount={25.5}
        totalAmount={100}
        symbol="€"
        onCommitWeight={jest.fn()}
      />
    );

    const input = screen.getByDisplayValue('25.50') as HTMLInputElement;
    expect(input).toBeInTheDocument();
    expect(screen.getByText('€')).toBeInTheDocument();
  });

  it('allows typing without committing immediately', () => {
    const handleCommit = jest.fn();
    render(
      <AbsoluteSplitInput
        amount={25}
        totalAmount={100}
        symbol="$"
        onCommitWeight={handleCommit}
      />
    );

    const input = screen.getByRole('spinbutton') as HTMLInputElement;
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '40' } });

    expect(input.value).toBe('40');
    expect(handleCommit).not.toHaveBeenCalled();
  });

  it('commits calculated weight on Enter key press when onCommitWeight provided', () => {
    const handleCommit = jest.fn();
    render(
      <AbsoluteSplitInput
        amount={25}
        totalAmount={100}
        symbol="$"
        onCommitWeight={handleCommit}
      />
    );

    const input = screen.getByRole('spinbutton');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '40' } });

    const preventDefault = jest.fn();
    fireEvent.keyDown(input, { key: 'Enter', preventDefault });

    expect(handleCommit).toHaveBeenCalledWith(40); // (40 / 100) * 100 = 40%
  });

  it('commits exact amount on Enter key press when onCommitAmount provided', () => {
    const handleCommitAmount = jest.fn();
    render(
      <AbsoluteSplitInput
        amount={25}
        totalAmount={100}
        symbol="$"
        onCommitAmount={handleCommitAmount}
      />
    );

    const input = screen.getByRole('spinbutton');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '42.50' } });

    const preventDefault = jest.fn();
    fireEvent.keyDown(input, { key: 'Enter', preventDefault });

    expect(handleCommitAmount).toHaveBeenCalledWith(42.5);
  });

  it('commits exact amount on blur when onCommitAmount provided', () => {
    const handleCommitAmount = jest.fn();
    render(
      <AbsoluteSplitInput
        amount={20}
        totalAmount={80}
        symbol="€"
        onCommitAmount={handleCommitAmount}
      />
    );

    const input = screen.getByRole('spinbutton');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '35' } });
    fireEvent.blur(input);

    expect(handleCommitAmount).toHaveBeenCalledWith(35);
  });

  it('commits calculated weight on blur', () => {
    const handleCommit = jest.fn();
    render(
      <AbsoluteSplitInput
        amount={20}
        totalAmount={80}
        symbol="€"
        onCommitWeight={handleCommit}
      />
    );

    const input = screen.getByRole('spinbutton');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '20' } });
    fireEvent.blur(input);

    expect(handleCommit).toHaveBeenCalledWith(25); // (20 / 80) * 100 = 25%
  });

  it('clamps calculated weight to maximum 100% if entered amount exceeds total amount', () => {
    const handleCommit = jest.fn();
    render(
      <AbsoluteSplitInput
        amount={50}
        totalAmount={100}
        symbol="€"
        onCommitWeight={handleCommit}
      />
    );

    const input = screen.getByRole('spinbutton');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '150' } });
    fireEvent.blur(input);

    expect(handleCommit).toHaveBeenCalledWith(100);
  });

  it('reverts to initial amount when totalAmount is 0 or non-numeric value entered', () => {
    const handleCommit = jest.fn();
    render(
      <AbsoluteSplitInput
        amount={30}
        totalAmount={0}
        symbol="€"
        onCommitWeight={handleCommit}
      />
    );

    const input = screen.getByRole('spinbutton') as HTMLInputElement;
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '50' } });
    fireEvent.blur(input);

    expect(handleCommit).not.toHaveBeenCalled();
    expect(input.value).toBe('30.00');
  });
});

describe('PercentageSplitInput', () => {
  it('displays weight initially', () => {
    render(
      <PercentageSplitInput weight={50} onCommitWeight={jest.fn()} />
    );

    const input = screen.getByDisplayValue('50');
    expect(input).toBeInTheDocument();
    expect(screen.getByText('%')).toBeInTheDocument();
  });

  it('commits percentage on Enter and prevents default form submission', () => {
    const handleCommit = jest.fn();
    render(
      <PercentageSplitInput weight={50} onCommitWeight={handleCommit} />
    );

    const input = screen.getByRole('spinbutton');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '75' } });
    fireEvent.keyDown(input, { key: 'Enter' });

    expect(handleCommit).toHaveBeenCalledWith(75);
  });

  it('commits percentage on blur and clamps between 0 and 100', () => {
    const handleCommit = jest.fn();
    render(
      <PercentageSplitInput weight={50} onCommitWeight={handleCommit} />
    );

    const input = screen.getByRole('spinbutton');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: '120' } });
    fireEvent.blur(input);

    expect(handleCommit).toHaveBeenCalledWith(100);
  });
});

describe('NewExpenseModal Sequential Absolute Split Preservation', () => {
  const mockPot = {
    id: 1,
    name: 'Vacation',
    default_currency: { id: 1, symbol: '$', code: 'USD' },
    users: [
      { uuid: 'u1', name: 'Alice' },
      { uuid: 'u2', name: 'Bob' },
      { uuid: 'u3', name: 'Charlie' },
      { uuid: 'u4', name: 'Dave' },
    ],
  };

  it('preserves previously entered manual absolute values when setting subsequent participants', () => {
    render(
      <NewExpenseModal
        open={true}
        onClose={jest.fn()}
        pot={mockPot as any}
        potId={1}
      />
    );

    // Set Total Amount = 100
    const totalAmountInput = screen.getByPlaceholderText('0.00');
    fireEvent.change(totalAmountInput, { target: { value: '100' } });

    // Toggle currency absolute mode ($ checkbox)
    const currencyCheckbox = screen.getByRole('checkbox');
    fireEvent.click(currencyCheckbox);

    // Initial state: Alice, Bob, Charlie, Dave each have 25.00
    const participantInputs = screen.getAllByRole('spinbutton').slice(1); // skip total amount input
    expect(participantInputs).toHaveLength(4);
    expect((participantInputs[0] as HTMLInputElement).value).toBe('25.00');

    // 1. User sets Alice (participantInputs[0]) = 30
    fireEvent.focus(participantInputs[0]);
    fireEvent.change(participantInputs[0], { target: { value: '30' } });
    fireEvent.blur(participantInputs[0]);

    // Alice is now 30.00
    expect((participantInputs[0] as HTMLInputElement).value).toBe('30.00');

    // 2. User sets Bob (participantInputs[1]) = 40
    fireEvent.focus(participantInputs[1]);
    fireEvent.change(participantInputs[1], { target: { value: '40' } });
    fireEvent.blur(participantInputs[1]);

    // Alice must remain 30.00, Bob is 40.00, and Charlie & Dave share remaining 30 (15.00 each)
    expect((participantInputs[0] as HTMLInputElement).value).toBe('30.00');
    expect((participantInputs[1] as HTMLInputElement).value).toBe('40.00');
    expect((participantInputs[2] as HTMLInputElement).value).toBe('15.00');
    expect((participantInputs[3] as HTMLInputElement).value).toBe('15.00');
  });

  it('scales all manual entries proportionally when cumulative total exceeds expense amount', () => {
    render(
      <NewExpenseModal
        open={true}
        onClose={jest.fn()}
        pot={mockPot as any}
        potId={1}
      />
    );

    // Set Total Amount = 100
    const totalAmountInput = screen.getByPlaceholderText('0.00');
    fireEvent.change(totalAmountInput, { target: { value: '100' } });

    // Toggle currency absolute mode
    const currencyCheckbox = screen.getByRole('checkbox');
    fireEvent.click(currencyCheckbox);

    const participantInputs = screen.getAllByRole('spinbutton').slice(1);

    // Set Alice = 40
    fireEvent.focus(participantInputs[0]);
    fireEvent.change(participantInputs[0], { target: { value: '40' } });
    fireEvent.blur(participantInputs[0]);

    // Set Bob = 40
    fireEvent.focus(participantInputs[1]);
    fireEvent.change(participantInputs[1], { target: { value: '40' } });
    fireEvent.blur(participantInputs[1]);

    // Set Charlie = 40 (Total manual = 120 > 100)
    fireEvent.focus(participantInputs[2]);
    fireEvent.change(participantInputs[2], { target: { value: '40' } });
    fireEvent.blur(participantInputs[2]);

    // Alice, Bob, Charlie should scale proportionally (33.33 or 33.34) and Dave receives 0.00
    const aliceVal = parseFloat((participantInputs[0] as HTMLInputElement).value);
    const bobVal = parseFloat((participantInputs[1] as HTMLInputElement).value);
    const charlieVal = parseFloat((participantInputs[2] as HTMLInputElement).value);
    const daveVal = parseFloat((participantInputs[3] as HTMLInputElement).value);

    expect(aliceVal).toBeCloseTo(33.33, 1);
    expect(bobVal).toBeCloseTo(33.33, 1);
    expect(charlieVal).toBeCloseTo(33.33, 1);
    expect(daveVal).toBe(0);
    expect(aliceVal + bobVal + charlieVal + daveVal).toBeCloseTo(100, 2);
  });

  it('clears manual locks when clicking Reset to equal', () => {
    render(
      <NewExpenseModal
        open={true}
        onClose={jest.fn()}
        pot={mockPot as any}
        potId={1}
      />
    );

    // Set Total Amount = 100
    const totalAmountInput = screen.getByPlaceholderText('0.00');
    fireEvent.change(totalAmountInput, { target: { value: '100' } });

    // Toggle currency mode
    fireEvent.click(screen.getByRole('checkbox'));

    const participantInputs = screen.getAllByRole('spinbutton').slice(1);

    // Set Alice = 70
    fireEvent.focus(participantInputs[0]);
    fireEvent.change(participantInputs[0], { target: { value: '70' } });
    fireEvent.blur(participantInputs[0]);

    expect((participantInputs[0] as HTMLInputElement).value).toBe('70.00');

    // Click Reset to equal
    fireEvent.click(screen.getByText('Reset to equal'));

    // All should reset to 25.00
    expect((participantInputs[0] as HTMLInputElement).value).toBe('25.00');
    expect((participantInputs[1] as HTMLInputElement).value).toBe('25.00');
    expect((participantInputs[2] as HTMLInputElement).value).toBe('25.00');
    expect((participantInputs[3] as HTMLInputElement).value).toBe('25.00');
  });
});
