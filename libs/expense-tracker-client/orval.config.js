module.exports = {
  'expense-tracker-client': {
    input: '../../apps/expense_tracker/openapi.json',
    output: {
      target: './src/expense-tracker-client.ts',
      baseUrl: 'http://localhost:5000', // find a way of doing this for all different environments
    },
  },
};
