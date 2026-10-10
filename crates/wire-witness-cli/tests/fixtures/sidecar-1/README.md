# wire-witness.sidecar/1 compatibility fixtures

Spec 014 section 3.8. Every release that claims interface
`wire-witness.sidecar/1` must pass these fixtures unchanged. A fixture may be
added; changing or removing one is an interface major change
(`wire-witness.sidecar/2`).

- `start/*.jsonl`: accepted `start` lines. Each reaches `ready` and, after
  stdin closes, exactly one `finished`; the process exits 0.
- `refused/<code>.jsonl`: `start` input refused before `ready`. The process
  writes nothing to stdout, names `<code>` on stderr, creates no output
  directory, and exits 2.
- `members/<message>.txt`: members each message must carry. A later release
  may add optional members.

Placeholders: `@OUTPUT@` is a fresh absolute attempt directory under a private
parent, `@PORT@` a free loopback port, and `@BUSY_PORT@` a loopback port the
test holds bound.
