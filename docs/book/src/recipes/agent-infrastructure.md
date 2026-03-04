# Agent Infrastructure Recipes

Recipes for deploying and managing AI agents via the pforge/OpenClaw integration.
These recipes demonstrate forjar's native MCP server management and multi-agent
orchestration capabilities.

## #73 pforge MCP Server

Deploy a pforge-managed MCP server as a forjar resource. The MCP server is
declared in YAML and managed alongside infrastructure — agent deployment
becomes a converging infrastructure operation.

**Tier**: 1+2 | **Idempotency**: Strong

Key features:
- MCP server as a native forjar resource type
- pforge YAML config drives server deployment
- Health monitoring via forjar status

## #74 Agent Deployment

Multi-agent deployment recipe. Deploys multiple AI agents with configuration
management, health monitoring, and permission policies — all declared in YAML.

**Tier**: 1+2 | **Idempotency**: Strong

Key features:
- Agent configuration as file resources
- Service lifecycle management
- Permission policy enforcement
