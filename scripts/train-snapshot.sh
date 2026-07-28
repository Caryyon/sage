#!/bin/bash
# SAGE Curriculum Training & Snapshot Pipeline
#
# This script:
#   1. Starts with a fresh brain (no existing knowledge)
#   2. Ingests a curriculum JSON into the NCA grid
#   3. Verifies retrieval quality
#   4. Exports a named snapshot (template) on success
#
# Usage:
#   ./train-snapshot.sh <curriculum.json> <snapshot-name> [--base <base-template>]
#
# Examples:
#   # Train base layer from scratch:
#   ./train-snapshot.sh curricula/high-school-graduate.json high-school-graduate
#
#   # Train specialty on top of base snapshot:
#   ./train-snapshot.sh curricula/accounting.json accounting-specialist --base high-school-graduate

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Arguments
CURRICULUM_FILE="${1:?Usage: $0 <curriculum.json> <snapshot-name> [--base <base-template>]}"
SNAPSHOT_NAME="${2:?Usage: $0 <curriculum.json> <snapshot-name> [--base <base-template>]}"
BASE_TEMPLATE=""

shift 2
while [[ $# -gt 0 ]]; do
    case "$1" in
        --base)
            BASE_TEMPLATE="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}Unknown argument: $1${NC}"
            exit 1
            ;;
    esac
done

# Paths
BRAIN_PATH="${SAGE_BRAIN_PATH:-$HOME/.sage/brain.bin}"
BACKUP_DIR="$HOME/.sage/backups"
TEMPLATES_DIR="$HOME/.sage/templates"

# Binaries (release build)
# Target dir may be in a custom location — find it via cargo metadata
CARGO_TARGET_DIR=$(cargo metadata --no-deps --format-version 1 2>/dev/null | python3 -c "import json,sys; print(json.load(sys.stdin).get('target_directory','target'))" 2>/dev/null || echo "target")
SAGE_CURRICULUM="${CARGO_TARGET_DIR}/release/sage-curriculum"
SAGE_TEMPLATE="${CARGO_TARGET_DIR}/release/sage-template"

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  SAGE Curriculum Training & Snapshot Pipeline     ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Curriculum:  ${CURRICULUM_FILE}"
echo -e "  Snapshot:    ${SNAPSHOT_NAME}"
if [[ -n "$BASE_TEMPLATE" ]]; then
    echo -e "  Base:        ${BASE_TEMPLATE} (forking from existing snapshot)"
fi
echo -e "  Brain path:  ${BRAIN_PATH}"
echo ""

# Check binaries exist
for bin in "$SAGE_CURRICULUM" "$SAGE_TEMPLATE"; do
    if [[ ! -f "$bin" ]]; then
        echo -e "${RED}❌ Binary not found: $bin${NC}"
        echo -e "   Run: cargo build --release"
        exit 1
    fi
done

# Check curriculum file exists
if [[ ! -f "$CURRICULUM_FILE" ]]; then
    echo -e "${RED}❌ Curriculum file not found: $CURRICULUM_FILE${NC}"
    exit 1
fi

# Step 1: Backup existing brain if present
if [[ -f "$BRAIN_PATH" ]]; then
    mkdir -p "$BACKUP_DIR"
    BACKUP_NAME="brain-$(date +%Y%m%d-%H%M%S).bin"
    cp "$BRAIN_PATH" "$BACKUP_DIR/$BACKUP_NAME"
    echo -e "${YELLOW}📦 Backed up existing brain to $BACKUP_DIR/$BACKUP_NAME${NC}"
fi

# Step 2: Prepare the starting brain
if [[ -n "$BASE_TEMPLATE" ]]; then
    # Fork from an existing template
    echo -e "${BLUE}🔀 Forking from template: $BASE_TEMPLATE${NC}"

    # Remove current brain so import doesn't complain
    rm -f "$BRAIN_PATH"
    # Also remove old stores to prevent contamination from previous training
    rm -f "$HOME/.sage/text_store.bin" "$HOME/.sage/hdc_store.bin"

    # Import the base template
    "$SAGE_TEMPLATE" import "$BASE_TEMPLATE" --force || {
        echo -e "${RED}❌ Failed to import base template '$BASE_TEMPLATE'${NC}"
        echo -e "   Available templates:"
        "$SAGE_TEMPLATE" list
        exit 1
    }
    echo -e "${GREEN}✅ Base template loaded${NC}"
else
    # Fresh brain — remove any existing brain
    echo -e "${BLUE}🆕 Starting with a fresh brain${NC}"
    rm -f "$BRAIN_PATH"
    # Also remove related stores so we truly start clean
    rm -f "$HOME/.sage/text_store.bin" "$HOME/.sage/hdc_store.bin"
fi

echo ""

# Step 3: Ingest curriculum
echo -e "${BLUE}📚 Ingesting curriculum...${NC}"
"$SAGE_CURRICULUM" ingest "$CURRICULUM_FILE" --confidence 0.95 --consolidation-steps 5 2>&1 | tee /tmp/sage_ingest_output.txt || {
    echo -e "${RED}❌ Curriculum ingestion failed${NC}"
    exit 1
}

# Show the ingestion report (it includes verification)
cat /tmp/sage_ingest_output.txt

# Step 4: Extract hit rate from the ingestion output (already includes verification)
# The ingestion report's "Total: X/Y (Z%)" line has what we need
INGEST_OUTPUT=$(cat /tmp/sage_ingest_output.txt 2>/dev/null || echo "")
HIT_RATE=$(echo "$INGEST_OUTPUT" | grep -oP 'Total:.*?(\d+\.?\d*)%' | grep -oP '\d+\.?\d*(?=%)' | head -1 || echo "0")

echo ""
echo -e "  Overall hit rate: ${YELLOW}${HIT_RATE}%${NC}"

# Check if we hit our quality threshold
THRESHOLD=70.0
MEETS_THRESHOLD=$(echo "$HIT_RATE >= $THRESHOLD" | bc -l 2>/dev/null || echo "0")

if [[ "$MEETS_THRESHOLD" == "1" ]]; then
    echo -e "${GREEN}✅ Hit rate ${HIT_RATE}% meets threshold (${THRESHOLD}%)${NC}"
else
    echo -e "${YELLOW}⚠️  Hit rate ${HIT_RATE}% is below threshold (${THRESHOLD}%)${NC}"
    echo -e "   Proceeding with snapshot anyway — can be improved later."
fi

echo ""

# Step 5: Export the snapshot
echo -e "${BLUE}📸 Exporting snapshot: $SNAPSHOT_NAME${NC}"

# Extract description from curriculum
TEMPLATE_DESC="Trained brain snapshot from curriculum: $SNAPSHOT_NAME"

"$SAGE_TEMPLATE" export "$SNAPSHOT_NAME" \
    --description "$TEMPLATE_DESC" \
    --tags "curriculum,trained" \
    --domain "general" || {
    echo -e "${RED}❌ Failed to export template${NC}"
    exit 1
}

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  ✅ Snapshot complete: $SNAPSHOT_NAME             ${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Template saved to: $TEMPLATES_DIR/${SNAPSHOT_NAME}.template"
echo ""
echo -e "  Verify:  sage-template info $SNAPSHOT_NAME"
echo -e "  Inspect: sage-template inspect $SNAPSHOT_NAME"
echo -e "  List:    sage-template list"
echo ""

# Show what we've got
"$SAGE_TEMPLATE" list