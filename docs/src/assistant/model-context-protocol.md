# Model Context Protocol

Editsync uses the [Model Context Protocol](https://modelcontextprotocol.io/) to interact with [context servers](./context-servers.md):

> The Model Context Protocol (MCP) is an open protocol that enables seamless integration between LLM applications and external data sources and tools. Whether you're building an AI-powered IDE, enhancing a chat interface, or creating custom AI workflows, MCP provides a standardieditsync way to connect LLMs with the context they need.

Check out the [Anthropic news post](https://www.anthropic.com/news/model-context-protocol) and the [Editsync blog post](https://editsync.khulnasoft.com/blog/mcp) for an introduction to MCP.

## Try it out

Want to try it for yourself?

The following context servers are available today as Editsync extensions:

- [Postgres Context Server](https://github.com/editsync-extensions/postgres-context-server)

## Bring your own context server

If there's an existing context server you'd like to bring to Editsync, check out the [context server extension docs](../extensions/context-servers.md) for how to make it available as an extension.

If you are interested in building your own context server, check out the [Model Context Protocol docs](https://modelcontextprotocol.io/introduction#get-started-with-mcp) to get started.
