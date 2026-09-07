---
name: It made the wrong call
about: A window was taken back that should not have been, or a theft went through
labels: decision
---

**What happened**

Which application, and what were you doing at that moment (typing, clicking, away
from the keyboard)?

**What should have happened**

**The log line**

Turn on `record_everything = true` in `%APPDATA%\wsmf\config.toml`, restart, reproduce
it, and paste the matching line from `%APPDATA%\wsmf\focus.log`. The reason column is
the part that matters.

**Setup**

- Mode (watch / guard / strict):
- Windows version (`winver`):
- wsmf version:
