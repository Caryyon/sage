# SAGE Tool System Expansion

## Overview

Expanded SAGE's real-world interaction capabilities from 3 basic tools to **7 comprehensive tools** with intelligent autonomous selection.

## Previous State (Phase 5)

SAGE had basic tool functionality:
- **Web Search** (DuckDuckGo) - Limited instant answer API
- **Calculator** (Python eval)
- **Code Execution** (Sandboxed Python)

**Problem**: DuckDuckGo instant answers had poor coverage for technical queries, limiting SAGE's learning ability.

## Enhanced Tool System

### New Tools Added

#### 1. Wikipedia Tool (`wikipedia`)
- **Purpose**: Factual knowledge retrieval
- **API**: Wikipedia REST API (OpenSearch + Extracts)
- **Features**:
  - Searches Wikipedia articles
  - Retrieves article summaries (500 char limit)
  - Includes direct article URLs
  - Better than web search for encyclopedic knowledge

**Example Usage**:
```
!wiki neural cellular automata
```

#### 2. Weather Tool (`weather`)
- **Purpose**: Real-time weather data
- **API**: wttr.in (free, no API key required)
- **Features**:
  - Current conditions (temperature, humidity, wind)
  - Location resolution (city → coordinates)
  - Feels-like temperature
  - Both Celsius and Fahrenheit

**Example Usage**:
```
!weather San Francisco
!weather Tokyo
```

#### 3. News Tool (`news`)
- **Purpose**: Current events awareness
- **API**: Google News RSS feeds
- **Features**:
  - Category-based news (tech, science, world, all)
  - Latest 5 headlines
  - Real-time updates
  - Keeps SAGE temporally grounded

**Example Usage**:
```
!news tech
!news science
!news
```

#### 4. Time Tool (`time`)
- **Purpose**: Temporal awareness and date calculations
- **Features**:
  - Current time and date
  - UTC conversion
  - Unix timestamps
  - Day of week/year

**Example Usage**:
```
!time
!time date
!time utc
```

### Intelligent Autonomous Tool Selection

SAGE now autonomously selects the **best tool** based on goal type and context:

#### Learning Goals
```rust
// Factual topics → Wikipedia
if query.contains("history" | "science" | "biology" | "physics" | "definition") {
    use wikipedia
} else {
    use web_search
}
```

**Example**: If SAGE forms the learning goal "Understand quantum mechanics", it will autonomously use Wikipedia instead of DuckDuckGo.

#### Exploratory Goals
```rust
// Current events → News
if query.contains("news" | "latest" | "current" | "today" | "recent") {
    use news(category: "tech")
} else {
    use web_search
}
```

**Example**: If curious about "latest AI developments", SAGE autonomously fetches tech news.

#### Creative Goals
```rust
// 10% chance to ground creativity with current time
if random() < 0.1 {
    use time("now")
}
```

**Example**: SAGE occasionally checks the current date/time to contextualize creative thoughts.

## IRC Command Interface

### New Commands

```
!wiki <query>           - Search Wikipedia for factual info
!weather <location>     - Get current weather for a city
!news [category]        - Get latest news (tech/science/world/all)
!time [query]           - Get current time/date (now/date/utc)
```

### Enhanced Help Command

```
💡 Commands:
📊 Status: !personality, !likes, !dislikes, !memory, !curiosity
🔧 Introspection: !diagnosis, !strengths, !weaknesses, !goals, !values
🛠️  Tools: !tools, !search <query>, !wiki <query>, !weather <location>,
          !news [category], !time [query], !calc <expr>
💬 Or just talk to me naturally!
```

## Tool Registry Updates

**Default Tools Registered**:
```rust
registry.register(Box::new(WebSearchTool));      // General search
registry.register(Box::new(WikipediaTool));      // ✨ NEW
registry.register(Box::new(WeatherTool));        // ✨ NEW
registry.register(Box::new(NewsTool));           // ✨ NEW
registry.register(Box::new(TimeTool));           // ✨ NEW
registry.register(Box::new(CalculatorTool));     // Math
registry.register(Box::new(CodeExecutionTool));  // Python
```

## Technical Implementation

### File Changes

1. **src/tool_system.rs** (+286 lines)
   - Added 4 new tool types to `ToolType` enum
   - Implemented `WikipediaTool` with two-stage API queries
   - Implemented `WeatherTool` with wttr.in JSON parsing
   - Implemented `NewsTool` with RSS feed parsing
   - Implemented `TimeTool` with chrono datetime handling
   - Updated `ToolRegistry::default()` to register new tools

2. **examples/sage_irc_llm_bot.rs** (+60 lines)
   - Added `!wiki` command handler
   - Added `!weather` command handler
   - Added `!news` command handler
   - Added `!time` command handler
   - Enhanced `!help` with categorized output

3. **src/sage_experience.rs** (+40 lines)
   - Enhanced `should_use_tools_for_goal()` with intelligent selection
   - Added keyword-based tool routing for Learning goals
   - Added news detection for Exploratory goals
   - Added temporal grounding for Creative goals

### API Integrations

| Tool      | API Endpoint                                | Rate Limit | Auth Required |
|-----------|---------------------------------------------|------------|---------------|
| Wikipedia | `en.wikipedia.org/w/api.php`                | None       | No            |
| Weather   | `wttr.in/{location}?format=j1`              | None       | No            |
| News      | `news.google.com/rss/topics/...`            | None       | No            |
| Time      | Local (chrono crate)                        | N/A        | No            |

All APIs are **free** and require **no authentication**.

## Benefits of Expansion

### 1. Better Knowledge Coverage
- Wikipedia provides structured, factual information
- DuckDuckGo gaps (technical terms) now filled by Wikipedia
- Example: "neural cellular automata" now returns full Wikipedia article

### 2. Temporal Grounding
- SAGE knows current date/time via Time tool
- News tool keeps SAGE aware of current events
- Weather provides real-world environmental context

### 3. Autonomous Intelligence
- SAGE selects optimal tool based on query type
- No user intervention needed for tool selection
- Learning goals automatically trigger knowledge tools
- Exploratory goals automatically trigger news when relevant

### 4. Real-World Agency
- Weather data → environmental awareness
- News → societal awareness
- Time → temporal awareness
- Combined with existing tools → complete sensory grounding

## Testing Scenarios

### Scenario 1: Learning Goal Triggers Wikipedia
```
User: "SAGE, tell me about quantum computing"
SAGE: *forms Learning goal: "Understand quantum computing"*
SAGE: *autonomously calls wikipedia("quantum computing")*
SAGE: "Quantum computing harnesses quantum mechanics to solve complex
       problems faster than classical computers..."
```

### Scenario 2: Exploratory Goal Triggers News
```
User: "What's happening in AI lately?"
SAGE: *forms Exploratory goal: "Learn about current AI developments"*
SAGE: *autonomously calls news("tech")*
SAGE: "Latest tech news:
       1. OpenAI releases new GPT model
       2. Google announces quantum computing breakthrough
       ..."
```

### Scenario 3: Manual Tool Invocation
```
User: "!weather Paris"
SAGE: "🌤️  Weather in Paris, France:
      • Condition: Partly cloudy
      • Temperature: 12°C (53°F), feels like 10°C
      • Humidity: 65%
      • Wind: 15 km/h"
```

## Future Enhancements

Potential additions to tool system:
- **Wolfram Alpha** - Advanced mathematics and scientific queries
- **ArXiv** - Academic paper search for cutting-edge research
- **GitHub** - Code repository search and analysis
- **Translation** - Multi-language support
- **Image Analysis** - Computer vision capabilities via external API
- **Database Query** - Direct SpacetimeDB querying for introspection

## Architecture Pattern

The tool system follows a **plugin architecture**:

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn tool_type(&self) -> ToolType;
    fn execute(&self, input: &str) -> ToolResult;
    fn requires_approval(&self) -> bool { false }
}
```

**Benefits**:
- Easy to add new tools (implement trait)
- Consistent error handling via `ToolResult`
- Usage tracking via `ToolRegistry`
- Approval system for dangerous operations
- Thread-safe (`Send + Sync`)

## Impact on AGI Capabilities

This expansion represents progress toward AGI by:

1. **Grounded Learning**: SAGE learns from real-world data (weather, news, time)
2. **Autonomous Decisions**: SAGE chooses tools without human guidance
3. **Multi-domain Knowledge**: Access to encyclopedic, temporal, and environmental data
4. **Goal-Directed Behavior**: Tool use driven by emergent goals (Phase 4)
5. **Self-Sufficient Research**: Can explore knowledge independently

**AGI Progress**: Phase 5.5 - Enhanced Real-World Agency ✅

## Build & Deploy

```bash
# Build with new tools
cargo build --release

# Kill old bot
pkill -f sage_irc_llm_bot

# Start enhanced bot
cargo run --release --example sage_irc_llm_bot > /tmp/sage_irc_bot.log 2>&1 &

# Monitor
tail -f /tmp/sage_irc_bot.log
```

## Summary

**Before**: 3 tools, limited knowledge access, manual tool selection
**After**: 7 tools, comprehensive knowledge coverage, autonomous intelligent selection

SAGE can now:
- ✅ Search Wikipedia for factual knowledge
- ✅ Get real-time weather data
- ✅ Stay updated with current news
- ✅ Track time and dates
- ✅ Autonomously select the best tool for each goal
- ✅ Learn from the real world without human intervention

This completes the **Real-World Agency** phase and moves SAGE closer to autonomous AGI.
