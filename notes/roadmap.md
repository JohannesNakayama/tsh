
## Next Steps

- [ ] create nix package

- [ ] add logging/tracing

- [ ] develop a zettel:
    - [ ] entrypoint:
        - [x] find by embedding (search bar with result list for selection)
        - [ ] most recent (result list for selection)
        <!-- - [ ] select date range -->
    - [x] select zettel -> open and edit in neovim
    - [x] store new zettel with old zettel id as parent id

- [ ] promote thoughts to texts
    - [ ] new struct for texts (with titles, etc.)

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
