KNOWN BUGS
==========

[ ] empty buffers are extremely unstable / unpredictable
[ ] appending to the end of the line brings the cursor one character too far

--------

  TODO
========

[ ] 0, continual: Assorted vim commands
[x] 2, medium: Status bar + command gutter
  [ ] 2, medium: Command (`:`) mode
[ ] 2, medium: Visual (`v`) mode(s)
[ ] 2, medium: .toml config file
[ ] 1, 2-byte wide characters
  [ ] 1, medium: show whitespace
[ ] 1, difficult: Multiple buffers at the same time
  [ ] 1, difficult: Dynamic adding / moving / removing / hiding of buffers
[ ] 1, difficult: Investigate multithreading for syntax highlighting / other highlighting optimizations
[ ] 1, medium: Directory viewing buffer
[ ] 1, difficult: plugin system OR nice Rust api for source code editing (a la DWM)
[ ] 1, difficult: LSP support

--------

  DONE
========

[x] Can't move cursor to beginning of line
[x] ESCing back into normal mode crashes
[x] Typing in insert mode types one character before the cursor
[x] When pressing `Enter` in insert mode, the cursor moves to the wrong column

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
[x] 3, easy: actual error handling instead of implicit panics (`?`)
[x] 2, difficult: colors
[x] 2, medium: abstract keycodes from code, call generic commands instead
  [x] 2, medium: editable keymap
[x] 3, medium: Naive caching
[-] 3, medium: re-render / *re-highlight* only lines around / *below* cursor
[x] 2, medium: Delete (`d`) and Change (`c`) modes
[x] 1, easy: Abstract tui layout of buffer
