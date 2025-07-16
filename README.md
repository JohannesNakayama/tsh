# Tsh

*A simple tool to help you think.*

> [!WARNING]
> For now, consider forking the repository and make it your own if you want to use it.

> [!IMPORTANT]
> Work in progress, probably indefinitely...

This is my personal writing tool.
My notes often get very cluttered and I don't find notes, thoughts, and ideas that I want to continue developing.
This tool saves all my writing in a SQLite database.
Notes can be queried with embeddings search, so I can find things quickly.

Notes are stored as a directed acyclic graph.
Initial thoughts, off-the-cuff ideas, etc. are added as root notes.
When working on a note again, the updated note is saved as a child node to the previous note.

I intend to continuously add workflows.
Here are some ideas I want to try:

- *Remix*: Find several notes to adjacent topics, put them in one note, and continue working on that.
- *Mixin*: Start with a more developed thought, and "mix-in" some more notes.

For these two workflows, all the notes that go into the combined note become parent nodes to the new note in the DAG.

## Setup

On first start, the program will create a database in `${XDG_DATA_HOME}/tsh/zettelkasten.db`.
You can also change that to a custom location in a config file which is expected at `${XDG_CONFIG_HOME}/tsh/config.toml`.
See `example_config.toml` for reference.
The `data_dir` attribute is optional and it will default to the above location.
All of that is a bit hacky in the code, I'll improve that soon.

## Dependencies

You have to specify a provider, an embeddings model, and an API key in the config file so that embeddings can be calculated for notes.
I use ollama with `allminilm:latest` and it's working fine for me so far.

Furthermore, notes are opened in neovim in a separate process when you add or iterate them.
That's currently hardcoded because I use neovim.
I might expose `editor` as a configuration option in the future.

## Installation

There is a nix package provided in this repository.
You can install it from GitHub with `pkgs.fetchFromGitHub`.
I hope to find time to include more detailed instructions soon.

