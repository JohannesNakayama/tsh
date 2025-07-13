# Tsh

*A simple tool to help you think.*

> [!IMPORTANT]
> Work in progress, probably indefinitely...

This is my personal writing tool.
My notes often get very cluttered and I don't find notes, thoughts, and ideas that I want to continue developing.
This tool saves all my writing in a SQLite database.
Notes can be queried with embeddings search, so I can find things quickly.
Notes are stored as a directed acyclic graph.
Initial thoughts, off-the-cuff ideas, etc. are added as root notes.
When working on a note again, the updated note is saved as a child node to the previous note.

> [!NOTE]
> Right now, the project is very early stage. I'm already using it, but I'm changing things constantly, so expect breaking changes.

I intend to continuously add workflows.
Here are some ideas I want to try:

- *Remix*: Find several notes to adjacent topics, put them in one note, and continue working on that.
- *Mixin*: Start with a more developed thought, and "mix-in" some more notes.

For these two workflows, all the notes that go into the combined note become parent nodes to the new note.

## Setup

On first start, the program will create a database in `${XDG_DATA_HOME}/tsh/zettelkasten.db`.
You can also change that to a custom location in a config file.

The config file is expected at `${XDG_CONFIG_HOME}/tsh/config.toml`.
See `example_config.toml` for reference.
The `data_dir` attribute is optional and it will default to the above location.

## Dependencies

You have to either specify a provider, an embeddings model, and an API key in the config file or have ollama service running locally.
Also, the local ollama should provide an appropriate embeddings model.
I use `allminilm:latest` and it's working fine for me so far.

## Installation

There is a nix package provided in this repository.
You can install it from GitHub with `pkgs.fetchFromGitHub`.
I hope to find time to include more detailed instructions soon.

