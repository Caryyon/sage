# Proactive Communication Implementation - Integration Guide

## Status: IN PROGRESS

## What's Been Done:
1. ✅ Created `src/inner_thoughts.rs` - Thought representation and evaluation
2. ✅ Created `src/proactive_communication.rs` - Decision engine with social timing
3. ✅ Fixed chrono::Timelike import
4. ✅ Modules compile successfully
5. ✅ Added imports to `sage_discord_autonomous.rs`

## Next Steps for Integration:

### Step 1: Add Shared HTTP Client
- Add `Arc<StdMutex<Option<Arc<serenity::http::Http>>>>` to share HTTP client
- Pass to autonomous thread
- Set in `ready()` callback

### Step 2: Modify Autonomous Thread
- Initialize `ProactiveCommunication` instance
- In curiosity mode, evaluate thoughts with proactive comm
- Send DMs when `should_share()` returns true

### Step 3: User ID Configuration
- Add environment variable `DISCORD_USER_ID` for target user
- Default to known user ID: `169984115123855360` (caryyon)

### Step 4: DM Sending Logic
Create helper async function:
```rust
async fn send_proactive_dm(
    http: &Arc<Http>,
    user_id: UserId,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let user = user_id.to_user(http).await?;
    let dm_channel = user.create_dm_channel(http).await?;
    dm_channel.say(http, message).await?;
    Ok(())
}
```

### Step 5: Integration Point
In `sage_discord_autonomous.rs` line ~441-453 (curiosity mode):
- Create `InnerThought` from question
- Call `proactive_comm.should_share()`
- If true, spawn async task to send DM
- Use `tokio::runtime::Handle::current()` to bridge sync/async

## Code Location Map:
- Autonomous thread: `examples/sage_discord_autonomous.rs:410-461`
- Curiosity mode: Lines 441-453
- Ready callback: Lines 80-89
- Handler struct: Lines 26-34

## Testing Plan:
1. Set environment variables:
   - `DISCORD_TOKEN`
   - `DISCORD_USER_ID=169984115123855360`
2. Run bot: `make discord`
3. Wait for idle time to trigger curiosity mode (360s)
4. Check if proactive DM is sent
5. Verify social timing penalties work (time of day, frequency)
