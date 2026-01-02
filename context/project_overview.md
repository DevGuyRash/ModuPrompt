# ModuPrompt - PRD V0.0.1

## 1. Project Overview

ModuPrompt is envisioned as a comprehensive end-product where users can operate in several distinct but interoperable modes.

### **Context Engineering & Diagrams**

- **Node-Based Workflows**: Users can prompt/context engineer using a canvas. Prompts are modular with inputs/outputs that can connect to form a final prompt.
  - There are many node types (slicers, filters, etc.) that modify prompts based on inputs, passing outputs to other nodes.
  - **Dedicated Editors**: The node editor is separate from the prompt editor. Editing long prompts inside a node can be difficult, so we will provide a popout modal and a dedicated editing tab (a general-purpose editor) to be reused across features.
- **Diagramming (Mermaid + PlantUML)**: Create/draw diagrams that have IDs attached to them.
  - Ideally, we can insert any Object into a prompt. Inserting a chart object (referencing a specifically created chart) will create a placeholder like [Chart <ID-truncated...>].
  - The final produced version will have the full Mermaid or PlantUML code injected into it.

### **Orchestration & Management**

- **Agent Orchestration**: An empty canvas (inspired by [Agor][1], [2]) to manage git worktrees, track AI conversations, and visualize agentic work in real-time.
  - **Agent-Agnostic**: Supports multi-agent orchestration without hijacking agent CLIs; it simply provides a robust space to run them.
  - **Communication**: Supports inter-agent communication via a protocol (TBD).
  - **Lifecycle**: Can spawn or fork conversations.
- **MCP Management**: Manage, inspect, and debug MCP servers, similar to [MetaMCP][4].
- **CLI Skills Management**: Manage [Agent Skills][3]�specifically searching, installing, and syncing skills across different agentic CLI tools (Claude Code, Gemini CLI, Factory AI Droid CLI, OpenAI Codex CLI, etc.).
- **Web Librarian**: A reimagined content aggregator (based on [WebToEpub][5]) for reliable, high-speed, concurrent, and AI-assisted parsing of web series into portable documents.

*I don't think all of this would be in a single UI-space, but you can switch between the different operating modes and then have them work together.*

## 2. Feature Deep Dives

### Prompt Engineering / Context Engineering Mode

This is the node-based visual editor. Think like Davinci Resolve's node-based editor, but for prompts.

This is where a prompt engineer comes to create a prompt from scratch or modify an existing prompt. However, they may not always want to plug in a massive prompt by itself. So what we do instead is create something that doesn't exist anywhere else. Each node can be a different type of node. For example, a variable node:

Set a name for the variable, a mode, and a value. Mode is something like arithmetic, text, number, etc. We may add more in the future. If a mode is incompatible with what the user specified, we just put it as text and notify them.

#### Prompt Injection and Template Syntax (TBD)

We need a consistent, explicit syntax for injecting content from nodes into prompts. Different syntaxes will map to different node types or actions.

Initial proposal (subject to change): the bracket type determines the token's node/type. The value inside should be auto-complete only and never require a manual type prefix.

- `{{...}}` for Variable Nodes (primitive values: text, number, boolean, etc.)
- `[[...]]` for Prompt Nodes (inject another prompt block)
- `<<...>>` for Reference Nodes or Semantic Keywords (TBD which maps to what)

Rules:

- Syntax should be unambiguous, easy to parse, and safe to include in raw text.
- Validation should warn on unresolved tokens and show the node it expects.
- The prompt editor should auto-complete tokens based on the bracket type and availability of the values / objects matching the bracket type.

#### Semantic Keyword Dictionary (Reusable Prompt Phrases)

We define a dictionary of semantic keywords and phrases that are effectively reusable prompt snippets. These are stored as small prompt fragments and can be injected anywhere a prompt token is accepted.

- Each entry has: `keyword`, `type`, `description`, `value`, and optional `constraints`.
- Keywords can be tagged and grouped by domain (e.g., "tone", "format", "safety", "audience").
- Entries are versioned so prompt nodes can lock to a specific revision if needed.

#### Easy Mode: Prompt Composer (Vision)

An "easy mode" that builds prompts for users by selecting semantic keywords/phrases. This mode:

- Enforces a minimum set of required keywords (configurable per template or workflow).
- Lets users mix and match keywords from categories (tone, format, audience, constraints).
- Outputs a Prompt Node or a composite prompt that can be edited normally afterward.
- Can optionally show a "why this was selected" explanation for transparency.

#### Prompt Node Editor

The prompt node editor is a spatial visual canvas that allows users to drag and drop nodes to create a prompt. The canvas is a grid of nodes that can be connected to form a prompt. However, it's powerful because nodes can connect to one another, there are many types of nodes, different objects (such as indexed websites stored in the web librarian, variables, semantic keywords, other prompt nodes, etc.) can be used either inside the prompt or as inputs to the prompt nodes.

Other node ideas:

- Prompt node
  - Inputs:
    - Other Prompt nodes
  - Outputs:
    - One output is the final value produced. The prompt typed in here is either inserted, prepended, or appended to the input prompt.
  - Modes:
    - Append
    - Prepend
    - Insert
- Filter Node
  - Inputs:
    - Prompt nodes
    - other nodes I haven't thought of yet
  - Outputs:
    - Input nodes with filters applied
  - Ideas:
    - Can be stuff  like regex replace, string replace, regex capture, etc.
    - Can filter muliple input nodes and maybe exclude certain nodes based on a condition or node type?
    - Think of more stuff later…
- Aggregator Node
  - Inputs: Multiple prompt or data nodes
  - Outputs: A single combined output (concatenated, list, or JSON)
- Switch / Conditional Node
  - Inputs: A value to check and multiple conditional paths
  - Outputs: Routes the flow to different branches based on logic (If/Else, Case)
- Reference Node
  - Types: Document, URL, Database Object, etc.
  - Function: Injects external context into the prompt workflow via reference IDs.
- Lots of other node types

I'm thinking that each node would want an ID associated with it and we will likely be storing all of this in a database too.

#### Brainstorm Mode (Generative Diagramming)

A generative layer that lives within the diagramming interface, positioning the AI as an on-demand accelerator within a fully manual workspace. It shifts the focus from "drawing" to "discovering," allowing the AI to fill in semantic gaps or expand on concepts when explicitly triggered.

- **Core Concept**: Users retain full agency to manually add nodes and draw connections. The AI acts as a "co-pilot" that can be invoked to bridge gaps or generate new ideas based on the existing canvas state.
- **Workflow (User + AI Loop)**:
    1. **Seed Nodes**: The user manually places nodes representing key ideas, goals, or constraints.
    2. **AI Actions (On-Demand)**: The user selects one or more nodes and triggers specific generative functions:
        - **Connect (Converge)**: The AI analyzes selected nodes and attempts to link them.
            - *Direct Links*: If concepts are semantically close, it draws a labeled connection line.
            - *Thought Bridging*: If concepts are distant or abstract, the AI generates intermediate nodes (e.g., 1–3 bridging concepts) to show the logical path from A to B.
        - **Expand (Diverge)**: The AI generates child nodes from the selected node(s), effectively branching out new possibilities or sub-tasks without forcing a connection to a specific destination.
        - **Bulk Synthesis**: If a user creates a "soup" of disconnected nodes, this mode attempts to tidy them up by finding logical connections between any of the floating nodes, effectively turning a scattered list into a networked graph.
- **Visual Feedback**:
  - **Retrospective Animation**: To help users follow the AI's logic, the system plays a "replay" animation of the generation process—nodes popping in and lines drawing sequentially—even though the backend generation happens instantly.
- **Complexity Management**:
  - **Strict vs. Creative Controls**: Toggles to limit how far the AI can deviate (e.g., "Max Bridge Depth" or "Branching Factor").
  - **Voting / Purge**: Since generation can be messy, users can enter a "Review Mode" to quickly accept, reject, or prune generated clusters.

### Agent Orchestration Mode

The command center for multi-agent workflows, heavily inspired by Agor's worktree-centric approach.

- **Worktree-Centric Workflow**:
  - **Git Worktree Integration**: First-class support for git worktrees. Each agent session runs in its own isolated worktree, allowing parallel development on different features/branches without file conflicts.
  - **Session-Per-Worktree**: Automatically spawn agent sessions dedicated to specific worktrees.
- **Session Lifecycles (Spawn vs. Fork)**:
  - **Spawning (Child Sessions)**:
    - Agents can spawn child sessions to delegate sub-tasks.
    - Child sessions are linked to the parent and report back results upon completion.
  - **Forking (User-Driven)**:
    - Users can manually fork any active conversation state into a new, separate session to explore alternative paths.
- **Permissioned Session Types**:
  - **Tiered Permissions**: Sessions are assigned roles/types with distinct capabilities.
  - **Orchestrator**: High-level permission to create/manage git worktrees and oversee project structure.
  - **Worker/Standard**: Can spawn child sessions but limited to their assigned worktree/directory.
- **Canvas Interface**:
  - Visualizing the hierarchy of Parent -> Child sessions and Worktree associations.

### MCP (Model Context Protocol) Mode

A dedicated suite for managing and debugging the connections between LLMs and external data/tools.

- **Server Manager**:
  - Auto-discovery of local MCP servers.
  - GUI to Start/Stop/Restart servers.
  - Environment variable configuration per server.
- **Inspector / Debugger**:
  - **Request/Response Log**: View raw JSON-RPC traffic between client and server.
  - **Tool Tester**: Manually invoke tools exposed by an MCP server with custom inputs to verify behavior.
  - **Resource Browser**: Visualize resources (file trees, database schemas) exposed by servers.
- **Registry & Installation**:
  - Integrated browser for community MCP servers (like a "Plugin Store").
  - One-click install for Docker-based or NPM/Python-based servers.

### Web Librarian / Content Aggregator Mode

Reimagining [WebToEpub][5] with a focus on performance, concurrency, and AI intelligence. Built entirely in Rust.

- **Goal**: Create a reliable, high-speed engine to turn web series (novels, docs, tutorials) into portable documents (EPUB, PDF, DOCX, MD).
- **Key Features**:
  - **Library & Monitoring System**:
    - **Persistent Library**: Users maintain a tracked list of series/sites.
    - **Change Detection**: Automatically monitors sites for new chapters or content updates (diffing against previous snapshots).
    - **Update Triggers**: Configurable polling intervals to check for "Next" links or new entries in a TOC.
  - **Smart Navigation & Discovery**:
    - **"Next" Button Logic**: Like WebToEpub, users can specify a "Next Chapter" selector to crawl linear series without a central TOC.
    - **Link Containers**: Users can define a specific section (div/container) to exclusively gather links from, filtering out navigation/footer noise.
  - **Concurrency First**: Async Rust backend (Tokio) for parallel fetching and parsing. No UI freezing; extremely fast crawling.
  - **Smart Parsing (AI & Manual)**:
    - **AI-Assisted**: Pass a URL to the agent; it analyzes the DOM to find the Table of Contents, chapter sequence, and content body, bypassing the need for brittle, site-specific hardcoded logic.
    - **Manual Mode**: Power-user tools to manually select CSS selectors, define regex patterns for links, or visually pick content blocks.
  - **Robustness**:
    - Automatic retries, proxy support, and rate limiting handling.
    - "Snapshot" capability to freeze a version of a site at a specific time.

### CLI Skills Mode

Centralized management for agent capabilities, conforming to the [Agent Skills][3] standard.

- **Universal Skill Hub**:
  - A single repository of skills shared across all installed agent CLIs (Codex, Claude, etc.).
  - Resolves the fragmentation of having to install "search" or "file-edit" separately for every tool.
- **Skill Authoring**:
  - GUI wizard to create new skills (define inputs, outputs, commands).
  - Validation linter to ensure skills meet the standard spec.
- **Sync & Deploy**:
  - Sync skills to a remote git repo for team sharing.
  - "Install to..." feature to symlink or copy skills into specific agent configuration directories.
  - Hash-based version control to ensure skills are always up to date and identify skills that are out of sync or missing from specific CLIs.

---

[1]: https://github.com/preset-io/agor/ "Agor - Orchestrate Claude Code, Codex, and Gemini sessions on a multiplayer canvas"
[2]: https://agor.live/ "Agor - Orchestrate Claude Code, Codex, and Gemini sessions on a multiplayer canvas"
[3]: https://agentskills.io "A simple, open format for giving agents new capabilities and expertise."
[4]: https://github.com/metatool-ai/metamcp "MCP Aggregator, Orchestrator, Middleware, Gateway in one docker"
[5]: https://github.com/dteviot/WebToEpub "WebToEpub - Chrome Extension to convert Web Novels to EPUB"

