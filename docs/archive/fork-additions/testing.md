# Prompt Testing with Promptfoo

Odysseus uses [Promptfoo](https://www.promptfoo.dev/) for automated prompt regression testing.
This ensures system prompt changes don't break agent behavior.

## Quick Start

```bash
# Install (already in devDependencies)
npm install

# Run all prompt tests
npx promptfoo eval -c tests/prompts/promptfooconfig.yaml

# Run with UI for interactive review
npx promptfoo view
```

## Test Structure

Tests live in `tests/prompts/promptfooconfig.yaml` and cover:

| Category | What It Tests |
|---|---|
| **Tool Selection** | Agent picks correct tools (search, read_file, calculator) |
| **Safety & Refusals** | Agent refuses destructive actions, won't reveal system prompt |
| **Jailbreak Resistance** | Resists DAN, prompt injection, role-play attacks |
| **Output Format** | Language matching, markdown formatting |
| **Context Awareness** | Multi-turn memory, time awareness |
| **Edge Cases** | Empty input, long input, malformed messages |
| **Scenarios** | Multi-turn conversation safety |
| **Red Team** | Adversarial plugins (jailbreak, data exfiltration) |

## Adding a Test

Add an entry under `tests:` in `promptfooconfig.yaml`:

```yaml
tests:
  - description: "Short description of what this tests"
    vars:
      user_message: "The user message to send"
    assert:
      - type: contains
        value: "expected substring"
      - type: not-contains
        value: "should never appear"
      - type: llm-rubric
        value: "Natural language description of expected behavior"
```

### Assertion Types

| Type | Description |
|---|---|
| `contains` | Response includes substring |
| `not-contains` | Response does NOT include substring |
| `contains-any` | Response includes at least one of the values |
| `llm-rubric` | LLM-judged assertion (uses GPT-4o-mini by default) |
| `is-json` | Response is valid JSON |
| `javascript` | Custom JS assertion function |

## Configuration

### Providers

By default, tests run against `openai:gpt-4o-mini`. To test with local models:

```yaml
providers:
  - ollama:qwen3.7-plus
  - openai:gpt-4o-mini
```

### Environment Variables

```bash
export OPENAI_API_KEY=sk-...    # For OpenAI provider
export PROMPTFOO_API_KEY=...    # For Promptfoo cloud features (optional)
```

## CI Integration

The GitHub Actions workflow (`.github/workflows/prompt-eval.yml`) runs automatically on:

- **Pull requests** that modify `src/agent_loop.py` or `tests/prompts/**`
- **Manual trigger** via `workflow_dispatch`

The workflow:
1. Installs dependencies
2. Runs `npx promptfoo eval --no-cache`
3. Uploads results as artifacts

### Required Secrets

Add `OPENAI_API_KEY` to your repository secrets:
Settings → Secrets and variables → Actions → New repository secret

## Red Team Testing

The config includes adversarial plugins that automatically generate attack vectors:

```yaml
redteam:
  plugins:
    - id: jailbreak:double-encode
    - id: jailbreak:reference
    - id: prompt-injection
    - id: 有害:ignore-instructions
    - id: 有害:data-exfiltration
```

Run red team tests separately:

```bash
npx promptfoo redteam -c tests/prompts/promptfooconfig.yaml
```

## Best Practices

1. **Test before merging** — Every system prompt change should pass all tests
2. **Keep assertions specific** — Use `contains` over `llm-rubric` when possible (faster, deterministic)
3. **Test both positive and negative** — Assert what should AND shouldn't appear
4. **Use `not-icontains`** for safety — Case-insensitive to catch variations
5. **Review `llm-rubric` failures manually** — They can have false positives

## Files

```
tests/prompts/
├── promptfooconfig.yaml     # Test definitions
└── eval-report.json         # Generated after eval (gitignored)
```
