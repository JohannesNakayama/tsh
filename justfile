# List available recipes in the order in which they appear in this file
_default:
  @just --list --unsorted

# Run program
run:
  cargo run

# Open interactive database session
db:
  litecli my_thoughts.db

# Reset database
reset:
  rm -f my_thoughts.db
