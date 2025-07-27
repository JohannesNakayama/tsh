## Next Steps

- [ ] add articles directly

- [ ] mix-in workflow
    - [ ] choose a (leaf node?) zettel
    - [ ] fill a list of zettels to mix-in (all root nodes?)
    - [ ] put everything in a buffer, save with parent_ids after editing

- [ ] tags and releases
    - [ ] build in CI
    - [ ] github release
    - [ ] publish with tags

- [ ] use flake-parts or manual setup in flake instead of flake-utils?

- [ ] check impurity in nix package definition -> can it be removed?

- [ ] how are migrations packaged when compiling the program? do I have to put them into the data dir too?

- [ ] get timestamps as u128 instead of i64

- [ ] add logging/tracing

- [ ] promote zettels to articles
    - [ ] implement functionality in tui

- [ ] function for remixing only root ideas (otherwise the same idea could end up in a thought twice)

- [ ] add tagging
    - [ ] automatic tagging with ollama model
    - [ ] manual tagging for organization

- [ ] store thought on save of vim buffer
    - [ ] keep buffer open, then save again on close

- [ ] fix flickering when entering/exiting neovim
    - [ ] embedded neovim instance with nvim-rs


## Concepts

Zettel workflows:

- add
- iterate
- remix
- mixin (special case of remix)


## Ideas

- add tags (auto tag?)
- add sources
    - automatically scrape, describe, embed websites
    - load from DOI
    - add manually (books, articles, etc.)
