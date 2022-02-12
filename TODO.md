KNOWN BUGS
==========

--------

  TODO
========

[ ] 3, easy: actual error handling instead of implicit panics (`?`)
[ ] 2, medium: Delete (`d`) and Change (`c`) modes
[ ] 2, medium: Visual (`v`) mode(s)
[ ] 2, medium: Status bar + command gutter
[ ] 2, medium: .toml config file
[ ] 2, medium: abstract keycodes from code, call generic commands instead
  [ ] 2, medium: editable keymap
[ ] 2, difficult: colors
[ ] 1, 2-byte wide characters
  [ ] 1, medium: show whitespace
[ ] 1, difficult: Multiple buffers at the same time
  [ ] 1, difficult: Dynamic adding / moving / removing / hiding of buffers
[ ] 1, medium: Directory viewing buffer
[ ] 1, difficult: plugin system OR nice Rust api for source code editing (a la DWM)
[ ] 1, difficult: LSP support

--------

  DONE
========

[x] Can't move cursor to beginning of line
[x] ESCing back into normal mode crashes
[x] Typing in insert mode types one character before the cursor

--------

[x] 3, medium: Use `ropey` crate instead of storing files as a Vec of lines
[x] 3, easy: LOGGING PLEASE
[x] 3, medium: Fix wacky behavior on last line of file
[x] 3, easy: refactor into separate files
[x] 3, switch to the `termion` backend instead of `crossterm`
[x] 3, medium: only render what you can see in the window; don't pass the entire file as a String
[x] 3, easy: Vertical scrolling
[x] 0, ???: Come up with a name (`gof`, `FeO`)
[x] 3, medium: revisit navigation code (`hjkl`, `aiAI`)
[x] 3, easy: Cursor navigation should ignore \n characters
[x] 3, easy: Save edited file
