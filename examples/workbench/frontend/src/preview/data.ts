export const exampleDefinitions = [
  ['page-and-configuration', 'Page and configuration', ['management'], 'Open the page or call its local echo route.'],
  ['local-model', 'Local Echo model', ['models', 'executor'], 'Send a normal Responses request to demo-echo.'],
  ['demo-accounts', 'Demo accounts', ['authentication', 'account_management'], 'Prepare the two demo accounts, then use the host account actions.'],
  ['quota-and-billing', 'Quota and billing', ['quota', 'billing'], 'Query demo quota and complete one Echo request.'],
  ['profiles-and-maintenance', 'Profiles and maintenance', ['request_profile', 'maintenance'], 'Select a demo profile and wait for the host maintenance worker.'],
  ['request-processing', 'Request processing', ['middleware'], 'Send a bound request with the workbench marker.'],
  ['routing-and-scheduling', 'Routing and account scheduling', ['model_router', 'scheduler'], 'Request demo-auto after preparing both demo accounts.'],
  ['request-observation', 'Request observation', ['request_lifecycle', 'usage', 'web_socket_observer'], 'Complete an Echo request and separately send a real WebSocket request.'],
  ['command-line', 'Command line', ['command_line'], 'Run the plugin ping command in the host process.'],
  ['client-authentication', 'Client authentication', ['frontend_authentication'], 'Bind the authenticator in a test environment.'],
] as const
