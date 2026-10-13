# Configuration

For basic configuration instructions, see [this documentation](https://developers.openai.com/codex/config-basic).

For advanced configuration instructions, see [this documentation](https://developers.openai.com/codex/config-advanced).

For a full configuration reference, see [this documentation](https://developers.openai.com/codex/config-reference).

## WebSocket fallback recovery

When a provider with `supports_websockets = true` exhausts its existing WebSocket
retry budget, Codex continues working normally over HTTP. The next eligible
request can try WebSocket again after a 60-second cooldown. If recovery fails,
the next cooldowns are 5 minutes and 15 minutes, followed by repeated 30-minute
cooldowns with no total attempt limit. Cooldowns never block HTTP requests.

Only a real WebSocket generation that receives `response.completed` restarts this
sequence at 60 seconds. A successful handshake, prewarm, HTTP response, partial
stream, or cancelled request does not. Recovery runs at request boundaries,
preserves active requests, and adds no inference probes. Fallback clears cached
WebSocket continuation state so recovery cannot reuse a stale `previous_response_id`.

These defaults require no new configuration. Providers with WebSocket support
disabled continue to use HTTP.

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.
