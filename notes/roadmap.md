
## Next Steps

- [ ] config
    - [ ] put default config (if one doesn't exist) into $XDG_CONFIG_HOME/tsh/config.toml on first startup (use `directories` crate)
    - [ ] put database into ~/.local/share/tsh/ (incl. migrations!)
    - [ ] create an isolated dev environment (to not mess with existing zettelkasten)

- [ ] add logging/tracing

- [ ] new develop zettel workflow:
    - [ ] entrypoint: most recent (result list for selection)

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
