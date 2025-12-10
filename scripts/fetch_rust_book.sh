#!/bin/bash
# Fetches The Rust Programming Language book and formats it for SAGE's library
# The book is open source (MIT/Apache 2.0) from https://github.com/rust-lang/book

BOOK_DIR="books"
OUTPUT_FILE="$BOOK_DIR/the_rust_programming_language.txt"

echo "📚 Fetching The Rust Programming Language book..."

# Create books dir if needed
mkdir -p "$BOOK_DIR"

# Create the header
cat > "$OUTPUT_FILE" << 'HEADER'
The Rust Programming Language
Steve Klabnik and Carol Nichols
Programming / Rust
The official book on the Rust programming language, covering ownership, borrowing, lifetimes, and more.
---
HEADER

# Fetch chapters from the raw GitHub source
BASE_URL="https://raw.githubusercontent.com/rust-lang/book/main/src"

# List of chapters to fetch (main content)
CHAPTERS=(
    "ch01-00-getting-started.md"
    "ch01-01-installation.md"
    "ch01-02-hello-world.md"
    "ch01-03-hello-cargo.md"
    "ch02-00-guessing-game-tutorial.md"
    "ch03-00-common-programming-concepts.md"
    "ch03-01-variables-and-mutability.md"
    "ch03-02-data-types.md"
    "ch03-03-how-functions-work.md"
    "ch03-04-comments.md"
    "ch03-05-control-flow.md"
    "ch04-00-understanding-ownership.md"
    "ch04-01-what-is-ownership.md"
    "ch04-02-references-and-borrowing.md"
    "ch04-03-slices.md"
    "ch05-00-structs.md"
    "ch05-01-defining-structs.md"
    "ch05-02-example-structs.md"
    "ch05-03-method-syntax.md"
    "ch06-00-enums.md"
    "ch06-01-defining-an-enum.md"
    "ch06-02-match.md"
    "ch06-03-if-let.md"
    "ch07-00-managing-growing-projects-with-packages-crates-and-modules.md"
    "ch08-00-common-collections.md"
    "ch08-01-vectors.md"
    "ch08-02-strings.md"
    "ch08-03-hash-maps.md"
    "ch09-00-error-handling.md"
    "ch09-01-unrecoverable-errors-with-panic.md"
    "ch09-02-recoverable-errors-with-result.md"
    "ch09-03-to-panic-or-not-to-panic.md"
    "ch10-00-generics.md"
    "ch10-01-syntax.md"
    "ch10-02-traits.md"
    "ch10-03-lifetime-syntax.md"
)

echo "📖 Downloading chapters..."
for chapter in "${CHAPTERS[@]}"; do
    echo "  - $chapter"
    curl -s "$BASE_URL/$chapter" >> "$OUTPUT_FILE"
    echo -e "\n\n" >> "$OUTPUT_FILE"
done

# Clean up markdown artifacts that don't read well as plain text
sed -i '' 's/^#/\n/g' "$OUTPUT_FILE" 2>/dev/null || sed -i 's/^#/\n/g' "$OUTPUT_FILE"

# Count pages (roughly 2000 chars each)
CHARS=$(wc -c < "$OUTPUT_FILE")
PAGES=$((CHARS / 2000))

echo ""
echo "✅ Done! Saved to $OUTPUT_FILE"
echo "📊 Approximately $PAGES pages for SAGE to read"
echo ""
echo "Restart the Discord bot to load the new book!"
