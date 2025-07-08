# List available recipes in the order in which they appear in this file
_default:
  @just --list --unsorted

# Run program
run:
  cargo run

# Open interactive database session
db:
  litecli --liteclirc ./litecli-config zettelkasten.db

# Reset database
reset:
  rm -f zettelkasten.db
