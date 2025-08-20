Context is critical - never guess how to use APIs or libraries.
Use context7 MCP liberally to get up-to-date documentation.
If that fails, use web search to find the correct usage.
Don't make assumptions about APIs or function signatures.
We are targeting android only, not desktop.
Do not try to mock out functions, we should be fixing compilation errors.
Use Android screenshots liberally to help debug problems with the frontend and iterate.
When building, use the predefined `npm run` scripts. If they dont' work, update them. Avoid calling builds directly, as we want to be able to use them again.
If you rebuild, you might need to grant permissions using the scripts available.
