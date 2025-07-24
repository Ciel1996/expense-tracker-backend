module.exports = {
  'expense-tracker-client': {
    input: '../../openapi/expense_tracker_openapi.json',
    output: {
      target: './src/expense-tracker-client.ts',
      baseUrl: 'http://localhost:3001', // find a way of doing this for all different environments
    },
  },
};
