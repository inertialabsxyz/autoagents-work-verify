# Autoagents Work Verify

A demonstration project showcasing the `autoagents` Rust library with a worker-verifier pattern for solving and validating math problems.

## Overview

This project demonstrates how to build AI agents using the `autoagents` framework. It implements two agents that work together:

1. **Worker Agent** - Solves math problems using a calculator tool
2. **Verifier Agent** - Independently verifies the worker's answer and returns structured JSON

## Features

- Custom tool implementation (Calculator)
- Multi-agent workflow (Worker → Verifier)
- Structured JSON output with pretty-printing
- Built on OpenAI's GPT-4o model

## Prerequisites

- Rust (2024 edition or later)
- OpenAI API key

## Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd autoagents-work-verify
```

2. Set up your OpenAI API key:
```bash
export OPENAI_API_KEY="your-api-key-here"
```

Or create a `.env` file:
```
OPENAI_API_KEY=your-api-key-here
```

3. Build the project:
```bash
cargo build
```

## Usage

Run the program:
```bash
cargo run
```

## Example Output

```
The calculator tool calculated the answer 140
The calculator tool calculated the answer 84
Worker returns: "The final price of the stock after increasing by 40% and then decreasing by 40% from an initial price of $100 is $84.00."

Verifier Result:
{
  "is_correct": true,
  "issues": [],
  "final_answer": "$84.00"
}
```

## How It Works

1. The **Worker Agent** receives a math problem
2. It uses the **Calculator Tool** to evaluate mathematical expressions
3. The Worker Agent returns its answer
4. The **Verifier Agent** independently solves the same problem
5. The Verifier compares its solution to the Worker's answer
6. Results are returned as structured JSON with:
   - `is_correct`: Boolean indicating if the answer is correct
   - `issues`: Array of any problems found
   - `final_answer`: The verified final answer

## Dependencies

- `autoagents` - AI agent framework
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization
- `meval` - Math expression evaluator
- `dotenv` - Environment variable loading

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
