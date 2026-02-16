#!/bin/bash
# Examples of using new generation control parameters

cd "$(dirname "$0")/.."

echo "=========================================="
echo "Example 1: Classic Russian Fairy Tale"
echo "=========================================="
cargo run -- generate \
  --genre "русская сказка" \
  --place "тридевятое царство" \
  --time "в стародавние времена" \
  --name "Иван-царевич" \
  --name "Василиса Прекрасная" \
  --moves 1 \
  --max-characters 3000 \
  --structure-only

echo ""
echo "=========================================="
echo "Example 2: Detective Noir"
echo "=========================================="
cargo run -- generate \
  --genre "detective noir" \
  --setting "1940s city" \
  --place "rainy Manhattan streets" \
  --time "winter 1947" \
  --name "Detective Mike Hammer" \
  --name "Femme Fatale" \
  --max-episodes 5 \
  --max-moments-per-episode 3 \
  --structure-only

echo ""
echo "=========================================="
echo "Example 3: Sci-Fi Short Story"
echo "=========================================="
cargo run -- generate \
  --genre "hard sci-fi" \
  --era "distant future" \
  --place "generation ship Aniara" \
  --time "year 2847" \
  --max-characters 2500 \
  --max-episodes 4 \
  --structure-only

echo ""
echo "=========================================="
echo "Example 4: Minimalist Horror"
echo "=========================================="
cargo run -- generate \
  --genre "psychological horror" \
  --tone "unsettling, minimalist" \
  --place "abandoned hospital" \
  --max-episodes 3 \
  --max-moments-per-episode 2 \
  --max-characters 2000 \
  --structure-only

echo ""
echo "=========================================="
echo "Example 5: Fantasy with Multiple Characters"
echo "=========================================="
cargo run -- generate \
  --genre "epic fantasy" \
  --setting "magical realm" \
  --place "enchanted forest" \
  --name "Aldric the Brave" \
  --name "Morgana the Wise" \
  --name "Shadowbane" \
  --moves 2 \
  --max-episodes 8 \
  --structure-only
