#!/bin/bash
# SAGE Hub Bootstrap — Populate the hub with starter specialist snapshots
#
# Usage: ./scripts/bootstrap-hub.sh [--hub http://localhost:3001]
#
# This script:
#   1. Ingests curriculum JSONs into trained brains
#   2. Exports brain templates
#   3. Defines specialist profiles
#   4. Publishes everything to the hub
#
# Prerequisites: sage-network-server running, sage binaries built

set -e

HUB="${1:-http://localhost:3001}"
CURRICULA_DIR="curricula"
SAGE_BIN="cargo run --release --bin"

echo "🧠 SAGE Hub Bootstrap"
echo "   Hub: $HUB"
echo ""

# Check if hub is reachable
echo "📡 Checking hub connectivity..."
if curl -s -o /dev/null -w "%{http_code}" "$HUB/health" | grep -q "200"; then
    echo "✅ Hub is running at $HUB"
else
    echo "⚠️  Hub not reachable at $HUB — starting in background..."
    echo "   Run: sage-network-server &"
    echo "   Then re-run this script."
    exit 1
fi

SPECIALISTS=(
    "junior-react-dev:Junior React Developer:Builds clean, tested React components with TypeScript and Tailwind:react,frontend,typescript,nextjs"
    "data-analyst:Data Analyst:Analyzes data with Python, SQL, and statistical methods:data,python,sql,statistics"
    "devops-engineer:DevOps Engineer:Manages infrastructure with Docker, K8s, Terraform, and CI/CD:devops,docker,kubernetes,terraform"
    "content-writer:Technical Content Writer:Writes clear documentation, tutorials, and developer content:writing,docs,markdown,devrel"
    "customer-support:Customer Support Specialist:Resolves issues with empathy and technical accuracy:support,customer-service,troubleshooting"
)

for SPEC in "${SPECIALISTS[@]}"; do
    IFS=':' read -r NAME DISPLAY TAGLINE TAGS <<< "$SPEC"

    CURRICULUM="$CURRICULA_DIR/$NAME.json"
    if [ ! -f "$CURRICULUM" ]; then
        echo "⚠️  No curriculum found for $NAME at $CURRICULUM — skipping"
        continue
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Processing: $DISPLAY"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Step 1: Ingest curriculum into a fresh brain
    echo "   [1/4] Ingesting curriculum..."
    $SAGE_BIN sage-curriculum -- ingest "$CURRICULUM" --confidence 0.95 --consolidation-steps 5 || {
        echo "   ⚠️  Curriculum ingestion failed for $NAME — skipping"
        continue
    }

    # Step 2: Export brain template
    echo "   [2/4] Exporting brain template..."
    $SAGE_BIN sage-template -- export "$NAME" \
        --description "Trained $DISPLAY specialist brain" \
        --tags "$TAGS" \
        --domain "$NAME" || {
        echo "   ⚠️  Template export failed for $NAME — skipping"
        continue
    }

    # Step 3: Define specialist profile
    echo "   [3/4] Defining specialist profile..."
    $SAGE_BIN sage-specialist -- define "$NAME" \
        --display-name "$DISPLAY" \
        --tagline "$TAGLINE" \
        --role "$NAME" \
        --template "$NAME" \
        --tags "$TAGS" \
        --rate 25 \
        --availability on-demand || {
        echo "   ⚠️  Specialist definition failed for $NAME — skipping"
        continue
    }

    # Step 4: Publish to hub
    echo "   [4/4] Publishing to hub..."
    $SAGE_BIN sage-specialist -- publish "$NAME" --hub "$HUB" || {
        echo "   ⚠️  Publish failed for $NAME — hub may not accept specialists yet"
        echo "   You can publish manually: sage-specialist publish $NAME --hub $HUB"
    }

    echo "   ✅ $DISPLAY complete!"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 Bootstrap complete!"
echo ""
echo "Hub URL: $HUB"
echo "Open in browser: $HUB/hub.html"
echo ""
echo "To hire a specialist:"
echo "  sage-specialist pull junior-react-dev --hub $HUB"
echo "  sage-specialist hire junior-react-dev --foreground --task 'Build a login form'"
echo ""
echo "To publish more:"
echo "  sage-specialist define <name> --role <preset>"
echo "  sage-specialist publish <name> --hub $HUB"
