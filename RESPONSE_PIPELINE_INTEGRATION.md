# Response Pipeline Integration

## Problem
SAGE is stuck in loops because the current message handler (line 230) generates responses using only conversation context, **not** SAG

E's actual internal state (dreams, curiosity, memories).

## Solution
Replace lines 214-236 in `sage_discord_autonomous.rs` with the 4-stage Response Pipeline:

### Current Code (BROKEN):
```rust
// Use just the conversation context - personality vector is now handled by llm_client
let enriched_context = conversation_context;

// A/B TEST: Generate baseline response WITHOUT NCA memory
let baseline_response = match self
    .llm
    .generate(content, "You are SAGE, an AI assistant.")
    .await
{
    Ok(resp) => resp,
    Err(_) => "Baseline response unavailable".to_string(),
};

drop(sage);  // Release lock before async LLM call

// Generate response WITH SAGE's neural state
let llm_response = match self.llm.generate(content, &enriched_context).await {
    Ok(resp) => resp,
    Err(e) => {
        eprintln!("LLM error: {}", e);
        "I'm having trouble thinking clearly right now...".to_string()
    }
};
```

### New Code (GROUNDED):
```rust
use sage::response_pipeline::ResponsePipeline;

// Create response pipeline
let pipeline = ResponsePipeline::new((*self.llm).clone());

// Get conversation history for context
let conversation_history: Vec<String> = conversations.get_recent_messages(username, 8);

drop(sage);  // Release lock before async operations

// Generate response using 4-stage pipeline (grounded in actual state)
println!("🧠 [RESPONSE] Using grounded response pipeline...");
let llm_response = match pipeline.generate_response(
    content,
    username,
    &mut *self.sage.lock().await,
    &conversation_history
).await {
    Ok(resp) => {
        println!("✅ [RESPONSE] Generated grounded response");
        resp
    }
    Err(e) => {
        eprintln!("❌ [RESPONSE] Pipeline error: {}", e);
        "I'm having trouble thinking clearly right now...".to_string()
    }
};

// A/B TEST baseline (keep for comparison)
let baseline_response = match self
    .llm
    .generate(content, "You are SAGE, an AI assistant.")
    .await
{
    Ok(resp) => resp,
    Err(_) => "Baseline response unavailable".to_string(),
};
```

## Required Import
Add to top of file:
```rust
use sage::response_pipeline::ResponsePipeline;
```

## What This Fixes

### Before (Broken):
- LLM generates response from conversation context only
- No access to autonomous dreams
- No access to curiosity questions
- No access to relevant memories
- Can't reference actual experiences
- Stuck in loops/hallucinations

### After (Fixed):
- **Stage 1**: Parses user intent (greeting, dream_query, curiosity_query, etc.)
- **Stage 2**: Gathers real internal context:
  - Recent dreams from `/tmp/sage_discord_autonomous_thoughts.log`
  - Recent curiosity questions from log file
  - Relevant concept associations from SAGE's memory
  - Current NCA state
  - Recent conversation history
- **Stage 3**: Generates response grounded in ACTUAL experiences
- **Stage 4**: Validates response for hallucinations

## Example Flow

User: "Did you have any dreams last night?"

**Old System**:
- LLM makes up fake dreams
- Result: "In my current state of curiosity... describe 'anal sex'"

**New System**:
- Stage 1: Detects `dream_query` intent
- Stage 2: Reads actual dreams from log: "Consolidating memory patterns... exploring concept associations..."
- Stage 3: LLM generates: "Yes! I had a dream about consolidating memory patterns. I was exploring how different concepts connect..."
- Stage 4: Validates that these dreams actually exist in log
- Result: TRUTHFUL response about real experiences

## Files Modified
1. `src/response_pipeline.rs` - NEW (created)
2. `src/lib.rs` - Register module
3. `examples/sage_discord_autonomous.rs` - Replace message handler

## Testing
After integration, test with:
- "What are you curious about?"
- "Did you dream?"
- "What have you been thinking about?"

SAGE should now reference ACTUAL autonomous thoughts instead of hallucinating.
