# Pagenor AI assistant

The native assistant offers Chat, Proofread Selection, Suggestions and Settings
in a docked or floating window. It supports an OpenAI-compatible service and
ChatGPT sign-in. Networking starts only through an explicit assistant action;
opening the app or its assistant does not contact a provider. Browser builds
retain persisted review commands but do not provide native networking/sign-in.

## Document access and review

Chat can list text frames, read body text and propose corrections in the active
document. Proofread Selection captures the selected text and gives the worker
only the proposal tool, processing up to 16,000 Unicode characters per batch.
Document text is treated as untrusted content. Model-provided offsets are discarded;
the host locates a unique exact quote in a captured snapshot.

Suggestions do not change text until the user chooses Accept or Accept All.
Locate selects the source passage; Reject preserves a rejected record. Accept All
validates every pending correction before applying one undo transaction. Overlapping,
stale, ambiguous or layout-control changes fail. Character/paragraph formatting,
frames and object markers are preserved. Editing or switching documents cancels a
running document task; late responses cannot mutate a different document. Stop is
cooperative, and blocked network workers have a four-worker ceiling and timeouts.

Chat, snapshots, suggestions and status persist in `.pgnr` documents. Credentials
are never included. Undo retains review history while restoring text and suggestion
status. New Conversation clears chat; Clear Records removes review data. Export
Report produces escaped standalone HTML and chooses a suffix when a file already
exists. Review records and reports intentionally contain document text.

## Settings and accounts

Choose a provider, endpoint and model in Settings. Compatible services accept an
HTTP(S) endpoint and optional API key. Local HTTP endpoints are supported; choose
an appropriate endpoint for the document content. Model lists preserve provider
ordering, labels and visibility. Save Settings, Load Models and Test Connection
are explicit actions.

Continue with ChatGPT opens browser consent using loopback callback, state,
nonce and PKCE, with signature/issuer/audience/expiry validation. Inference requires
`chatgpt.tokens.use.direct`, rather than identity-only access. The window explains
that ChatGPT requests use plan allowance and provides Manage Usage. Actual account
eligibility is determined by the provider. Settings store account labels and selected
model under the Pagenor configuration directory; secrets use macOS Keychain,
Windows credential storage or Linux Secret Service. Credential-store failures are
shown to the user. Linux requires an available, unlocked Secret Service.

Responses use `store:false` and streamed completion; tools are accepted only after a
complete response. Requests are bounded to 4 MiB, responses to 16 MiB, each task to
16 model rounds and 100 tool calls per response. Review limits are 4 MiB per story, 16 MiB total,
256 snapshots, 2,048 suggestions and 4 MiB chat. Limit failures preserve the document.
These limits can reject proofreading large selections with repeated story snapshots.

## Verification and provenance

Adapted locally from upstream closed PR #54 at
`21681fb3379ca55ef1c7498ecd19a8f74b16178f`, with original MIT OR Apache-2.0 terms.
Offline tests exercise both protocols using deterministic responses, Unicode exact
quotes, formatting preservation, atomic review/undo/save-load, stale/cancelled
workers, bounded records/requests and report collisions. Local rendered UI checks
cover review and settings. Upstream live-test claims are not Pagenor validation.
Actual browser sign-in, account refresh/revocation, live inference and native
Windows/Linux credential integration remain milestone checks.

Official protocol references checked for this adaptation:

- [Open-source sign-in](https://developers.openai.com/siwc/token-sharing-open-source/sign-in)
- [Models and inference](https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference)
- [Sign-in UI guidelines](https://developers.openai.com/siwc/ui-ux-guidelines)

Engine command families: `ai.text.*`, `ai.suggestion.*`, `ai.chat.*`,
`ai.task.status`, `ai.records.clear`, `ai.report.html`. Native UI commands add
`window.ai`, `window.ai.layout`, `ai.chat.send`, `ai.proofread.selection`,
`ai.task.stop`, `ai.settings.*`, `ai.auth.*` and `ai.report.export`.
See [control protocol](control-protocol.md).
