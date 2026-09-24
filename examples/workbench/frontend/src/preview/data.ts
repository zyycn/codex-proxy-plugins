export const exampleDefinitions = [
  ['page-and-configuration', 'Page and configuration', ['management'], 'Open the page or call its local echo route.'],
  ['request-processing', 'Request processing', ['middleware'], 'Send a bound request with the workbench marker.'],
  ['routing-and-scheduling', 'Routing and account scheduling', ['model_router', 'scheduler'], 'Send a marked request to an available model.'],
  ['request-observation', 'Request observation', ['request_lifecycle', 'usage', 'web_socket_observer'], 'Complete a model request and separately send a real WebSocket request.'],
  ['command-line', 'Command line', ['command_line'], 'Run the plugin ping command in the host process.'],
  ['client-authentication', 'Client authentication', ['frontend_authentication'], 'Bind the authenticator in a test environment.'],
] as const
