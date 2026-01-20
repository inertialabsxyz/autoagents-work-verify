use autoagents::async_trait;
use autoagents::core::agent::task::Task;
use autoagents::core::agent::{AgentBuilder, DirectAgent};
use autoagents::core::tool::{ToolCallError, ToolInputT, ToolRuntime, ToolT};
use autoagents::llm::backends::openai::OpenAI;
use autoagents::llm::builder::LLMBuilder;
use autoagents::prelude::ReActAgent;
use autoagents::prelude::SlidingWindowMemory;
use autoagents::prelude::{AgentOutputT, ReActAgentOutput};
use autoagents_derive::tool;
use autoagents_derive::{AgentHooks, ToolInput, agent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ToolInput, Debug)]
struct CalculatorArgs {
    #[input(description = "The mathematical expression to evaluate (e.g., '20 + 5')")]
    expression: String,
}

#[tool(name = "calculate", description = "Add two numbers", input = CalculatorArgs)]
struct CalculateTool;

#[async_trait]
impl ToolRuntime for CalculateTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let a: CalculatorArgs = serde_json::from_value(args)?;
        return match meval::eval_str(a.expression) {
            Ok(v) => Ok(v.into()),
            Err(e) => Err(ToolCallError::RuntimeError(Box::new(e))),
        };
    }
}

use autoagents_derive::AgentOutput;
#[derive(Debug, Serialize, Deserialize, AgentOutput)]
struct WorkerOut {
    #[output(description = "The result value")]
    value: f64,
}
impl From<ReActAgentOutput> for WorkerOut {
    fn from(out: ReActAgentOutput) -> Self {
        serde_json::from_str(&out.response).unwrap_or(WorkerOut { value: 0f64 })
    }
}

#[agent(
    name = "worker_agent",
    description = "Solve basic math using calculate tool and return a string",
    tools = [CalculateTool],
    output = WorkerOut
)]
#[derive(Clone, AgentHooks, Default)]
struct WorkerAgent;

#[agent(
    name = "verifier_agent",
    description = "You solve math problems. \
    Solve independently, compare to provided answer. \
    Report AGREE or DISAGREE with explanation",
    output = String
)]
#[derive(Clone, AgentHooks, Default)]
struct VerifierAgent;

fn extract_and_format_json(response: &str) -> String {
    // Try to extract JSON from markdown code fences
    let json_str = if let Some(start) = response.find("```json") {
        // Find the closing ``` after the opening ```json
        let search_start = start + 7;
        if let Some(end_offset) = response[search_start..].find("```") {
            let json_end = search_start + end_offset;
            &response[search_start..json_end].trim()
        } else {
            response
        }
    } else {
        response
    };

    // Try to parse and pretty-print the JSON
    match serde_json::from_str::<Value>(json_str) {
        Ok(json) => serde_json::to_string_pretty(&json).unwrap_or_else(|_| response.to_string()),
        Err(_) => response.to_string(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let llm: Arc<OpenAI> = LLMBuilder::<OpenAI>::new()
        .api_key(env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"))
        .model("gpt-4o")
        .build()?;

    let worker_agent = ReActAgent::new(WorkerAgent);
    let worker_handle = AgentBuilder::<_, DirectAgent>::new(worker_agent)
        .llm(llm.clone())
        .memory(Box::new(SlidingWindowMemory::new(10)))
        .build()
        .await?;

    let question = "A stock price increases by 40% on Monday, then decreases by 40% on Tuesday. \n
    If it started at $100, what is the final price?";

    let worker_out = worker_handle.agent.run(Task::new(question)).await?;

    println!("Worker returns: {:?}", worker_out);

    // Sequential, pass the result of Worker on to Verifier
    let verifier_agent = ReActAgent::new(VerifierAgent);
    let verifier_handle = AgentBuilder::<_, DirectAgent>::new(verifier_agent)
        .llm(llm)
        .memory(Box::new(SlidingWindowMemory::new(10)))
        .build()
        .await?;

    let verify_prompt = format!(
        r#"
    You are a strict verifier.

    User question:
    {q}

    Solver answer:
    {a}

    Tasks:
    1) Decide if the solver answer is correct.
    2) If incorrect or incomplete, correct it.
    3) Return ONLY valid JSON:
    {{
      "is_correct": boolean,
      "issues": [string, ...],
      "final_answer": string
    }}
    "#,
        q = question,
        a = worker_out.value
    );

    let verifier_out = verifier_handle.agent.run(Task::new(verify_prompt)).await?;

    println!("\nVerifier Result:");
    println!("{}", extract_and_format_json(&verifier_out));
    Ok(())
}
