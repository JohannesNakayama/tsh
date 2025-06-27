# List available recipes in the order in which they appear in this file
_default:
  @just --list --unsorted

# Run program
run:
  cargo run

# Apply database migrations
migrate:
  sqlx migrate run

# Open interactive database session
db:
  litecli my_thoughts.db

# Reset database
reset:
  sqlx database reset
