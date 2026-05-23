# Changelog

## [0.1.0] — 2026-05-22

### Added
- Full syntax highlighting for `.wurl` files
- Markdown fenced code block injection for ` ```wurl ` blocks
- Language configuration: line comments (`#`), auto-close quotes and brackets
- Indentation rules: increase after `group`/`test`, decrease on `end`
- Highlighting for all assert targets: `status`, `body`, `body.path`, `header.*`, `cookie.*`, `duration`
- All matcher keywords: `equals`, `contains`, `gt`, `gte`, `lt`, `lte`, `length`, `matches`, etc.
- `not` negation operator
- Case-insensitive HTTP method highlighting (`GET`/`get`/`Post`/…)
- Regex literal highlighting (`r"pattern"`)
- Body parameter highlighting (`body.key = value`)
