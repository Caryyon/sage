//! Tool System Demonstration
//!
//! This example demonstrates all 8 tools available in SAGE's tool system:
//! - WebSearch (DuckDuckGo)
//! - Wikipedia
//! - Weather (wttr.in)
//! - News (Google News RSS)
//! - Time/Date
//! - Calculator (Python eval)
//! - Code Execution (sandboxed Python)
//! - File Reading (with directory whitelisting)

use sage::tool_system::*;

fn main() {
    println!("🛠️  SAGE Tool System Demonstration\n");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Initialize tool registry with default tools
    let mut registry = ToolRegistry::default();

    println!("📋 Available Tools:\n");
    for (name, description) in registry.list_tools() {
        let requires_approval = if registry.requires_approval(name) {
            " [REQUIRES APPROVAL]"
        } else {
            ""
        };
        println!("  • {}{}", name, requires_approval);
        println!("    {}\n", description);
    }

    println!("═══════════════════════════════════════════════════════════════\n");

    // Test 1: Calculator Tool
    println!("🧮 TEST 1: Calculator\n");
    println!("   Query: (42 * 7) + 15");
    match registry.execute("calculator", "(42 * 7) + 15") {
        Ok(result) => {
            if result.success {
                println!("   ✅ Result: {}\n", result.output.trim());
            } else {
                println!("   ❌ Error: {:?}\n", result.error);
            }
        }
        Err(e) => println!("   ❌ Execution failed: {}\n", e),
    }

    // Test 2: Time Tool
    println!("⏰ TEST 2: Time & Date\n");
    println!("   Query: now");
    match registry.execute("time", "now") {
        Ok(result) => {
            if result.success {
                println!("   ✅ Current time:");
                for line in result.output.lines() {
                    println!("      {}", line);
                }
                println!();
            } else {
                println!("   ❌ Error: {:?}\n", result.error);
            }
        }
        Err(e) => println!("   ❌ Execution failed: {}\n", e),
    }

    // Test 3: Weather Tool
    println!("🌦️  TEST 3: Weather\n");
    println!("   Query: San Francisco");
    match registry.execute("weather", "San Francisco") {
        Ok(result) => {
            if result.success {
                println!("   ✅ Weather data:");
                for line in result.output.lines() {
                    println!("      {}", line);
                }
                println!();
            } else {
                println!("   ⚠️  Error: {:?}", result.error);
                println!("      (Weather API may be rate-limited or unavailable)\n");
            }
        }
        Err(e) => println!("   ❌ Execution failed: {}\n", e),
    }

    // Test 4: Wikipedia Tool
    println!("📚 TEST 4: Wikipedia\n");
    println!("   Query: Neural Cellular Automata");
    match registry.execute("wikipedia", "Neural Cellular Automata") {
        Ok(result) => {
            if result.success {
                println!("   ✅ Wikipedia extract:");
                // Truncate to first 200 chars for demo
                let preview = if result.output.len() > 300 {
                    format!("{}...", &result.output[..300])
                } else {
                    result.output.clone()
                };
                for line in preview.lines() {
                    println!("      {}", line);
                }
                println!();
            } else {
                println!("   ⚠️  Error: {:?}\n", result.error);
            }
        }
        Err(e) => println!("   ❌ Execution failed: {}\n", e),
    }

    // Test 5: Web Search Tool
    println!("🔍 TEST 5: Web Search\n");
    println!("   Query: Rust programming language");
    match registry.execute("web_search", "Rust programming language") {
        Ok(result) => {
            if result.success {
                println!("   ✅ Search results:");
                for line in result.output.lines() {
                    println!("      {}", line);
                }
                println!();
            } else {
                println!("   ⚠️  Error: {:?}", result.error);
                println!("      (DuckDuckGo API may have no instant answer)\n");
            }
        }
        Err(e) => println!("   ❌ Execution failed: {}\n", e),
    }

    // Test 6: News Tool
    println!("📰 TEST 6: News Headlines\n");
    println!("   Query: tech");
    match registry.execute("news", "tech") {
        Ok(result) => {
            if result.success {
                println!("   ✅ Latest tech news:");
                for line in result.output.lines().take(8) {
                    println!("      {}", line);
                }
                println!();
            } else {
                println!("   ⚠️  Error: {:?}", result.error);
                println!("      (News RSS feed may be unavailable)\n");
            }
        }
        Err(e) => println!("   ❌ Execution failed: {}\n", e),
    }

    // Test 7: Code Execution Tool (simple print)
    println!("🐍 TEST 7: Code Execution (REQUIRES APPROVAL)\n");
    println!("   Code: print('Hello from SAGE!')");
    match registry.execute("execute_code", "print('Hello from SAGE!')") {
        Ok(result) => {
            if result.success {
                println!("   ✅ Output: {}\n", result.output.trim());
            } else {
                println!("   ❌ Error: {:?}\n", result.error);
            }
        }
        Err(e) => println!("   ❌ Execution failed: {}\n", e),
    }

    // Test 8: Code Execution - Math
    println!("🧪 TEST 8: Code Execution - Computation\n");
    let fibonacci_code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n-1) + fib(n-2)

for i in range(10):
    print(f"fib({i}) = {fib(i)}")
"#;
    println!("   Code: Fibonacci sequence (first 10 numbers)");
    match registry.execute("execute_code", fibonacci_code) {
        Ok(result) => {
            if result.success {
                println!("   ✅ Output:");
                for line in result.output.lines() {
                    println!("      {}", line);
                }
                println!();
            } else {
                println!("   ❌ Error: {:?}\n", result.error);
            }
        }
        Err(e) => println!("   ❌ Execution failed: {}\n", e),
    }

    // Display usage statistics
    println!("═══════════════════════════════════════════════════════════════\n");
    println!("📊 Tool Usage Statistics:\n");
    let stats = registry.get_usage_stats();
    for (tool_name, count) in stats {
        if count > 0 {
            println!("   • {}: {} execution(s)", tool_name, count);
        }
    }

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("✅ Tool system demonstration complete!");
    println!("\n💡 Integration Ideas:");
    println!("   • Connect tools to Discord bot (e.g., !weather London)");
    println!("   • Let SAGE autonomously use tools during curiosity mode");
    println!("   • LLM-powered tool selection based on user questions");
    println!("   • Tool chaining (search → read → summarize)");
    println!("   • Add FileReadTool with safe directory whitelisting\n");
}
