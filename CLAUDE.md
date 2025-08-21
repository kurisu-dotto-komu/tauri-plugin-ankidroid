Context is critical - never guess how to use APIs or libraries.
Use context7 MCP liberally to get up-to-date documentation.
If that fails, use web search to find the correct usage.
Don't make assumptions about APIs or function signatures.
We are targeting android only, not desktop.
Use Android screenshots liberally to help debug problems with the frontend and iterate.
When building, use the predefined `npm run` scripts. If they dont' work, update them.
Avoid calling builds and tests directly, as we want to be able to use them again (use `npm run`).
If you rebuild, you might need to grant permissions using the scripts available.
You might need to run the dev server in the background if running the app in dev mode.
Do not take shortcuts like mocking out functions to get tests to pass - we should be fixing compilation errors properly.
