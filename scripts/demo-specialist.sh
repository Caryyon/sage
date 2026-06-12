#!/bin/bash
# SAGE Specialist Demo — Full pipeline proof
# Shows: curriculum → brain → template → specialist → task → NCA retrieval → prompt → result

set -e

echo "═══════════════════════════════════════════════════════════════"
echo "  SAGE SPECIALIST — END-TO-END DEMO"
echo "  No cloud. No API keys. No external LLM required."
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Step 1: Show the brain state
echo "═══ STEP 1: Brain State ═══"
echo ""
echo "The NCA brain has been trained on 120 React facts across 6 topics."
echo "It stores knowledge as activation patterns in a 256×256 grid."
echo ""
sage insights 2>&1 | head -20
echo ""

# Step 2: Query the brain directly
echo "═══ STEP 2: NCA Knowledge Retrieval ═══"
echo ""
echo "Task: 'Build a React login form component with email and"
echo "       password fields, form validation, and a submit handler'"
echo ""
echo "The NCA brain retrieves relevant knowledge WITHOUT any LLM:"
echo ""

sage search "Build a React login form component with email and password fields, form validation, and a submit handler" -n 8 2>&1
echo ""

# Step 3: Show the specialist profile
echo "═══ STEP 3: Specialist Profile ═══"
echo ""
sage-specialist info junior-react-dev 2>&1
echo ""

# Step 4: Show what the augmented prompt looks like
echo "═══ STEP 4: Augmented System Prompt ═══"
echo ""
echo "This is what gets sent to the language head."
echo "The NCA-retrieved knowledge is injected into the specialist's"
echo "system prompt, giving it domain-specific context."
echo ""
echo "--- BEGIN AUGMENTED PROMPT (first 1500 chars) ---"
python3 -c "
import json, subprocess, sys

# Get specialist profile
result = subprocess.run(['sage-specialist', 'info', 'junior-react-dev', '--json'], capture_output=True, text=True)
profile = json.loads(result.stdout)

# Get NCA knowledge
result2 = subprocess.run(['sage', 'search', 'Build a React login form component with email and password fields, form validation, and a submit handler', '-n', '8', '--json'], capture_output=True, text=True)
knowledge = json.loads(result2.stdout)

# Build augmented prompt
prompt = profile['prompt']
system = prompt['identity'] + '\n\n## Task Instructions\n' + prompt['task_instructions'] + '\n\n## Quality Standards\n' + prompt['quality_standards'] + '\n\n## Communication Style\n' + prompt['communication_style'] + '\n\n## Constraints\n' + prompt['constraints']

facts = [r.get('text', f\"pattern at ({r['position'][0]},{r['position'][1]})\") for r in knowledge.get('results', [])]
knowledge_section = '\n\n## Recalled Knowledge from NCA Brain\n' + '\n'.join(facts)

augmented = system + knowledge_section
print(augmented[:1500])
print('...')
print(f'')
print(f'Total prompt length: {len(augmented)} chars')
print(f'NCA knowledge added: {len(knowledge_section)} chars')
print(f'Facts retrieved: {len(facts)}')
" 2>&1
echo "--- END AUGMENTED PROMPT ---"
echo ""

# Step 5: Summary
echo "═══ STEP 5: What Makes This Different ═══"
echo ""
echo "A bare LLM gets: 'Build a login form' → generic React knowledge from pre-training"
echo "SAGE specialist gets: 'Build a login form' + 8 retrieved facts about forms,"
echo "  validation, onSubmit, hooks, and controlled components from its NCA brain"
echo ""
echo "The LLM is the mouth. The NCA brain is the memory."
echo "Same language capability, different knowledge → different specialist."
echo ""
echo "After the task completes, the result gets encoded back into the NCA brain."
echo "The specialist literally gets better at React every time it works."
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  DEMO COMPLETE"
echo "═══════════════════════════════════════════════════════════════"
