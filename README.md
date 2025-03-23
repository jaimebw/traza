# 🇪🇸 Traza (Trace)

A simple and powerful build logging utility for developers.

## What is Traza?

**Traza** (Spanish for "trace" or "track") is a lightweight command-line utility that captures and logs build outputs to a SQLite database. It's designed to help developers keep track of build processes, errors, and outputs across different projects.

## Features

- 📝 Capture stdin and store it in a structured database
- 🏷️ Tag logs with project names and custom tags
- 🔍 Easy retrieval of past build logs
- 💾 Persistent storage using SQLite
- 🚀 Minimal overhead for your build processes


## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/traza.git

# Build the project
cd traza
cargo build --release

# Install
make install

# Optional: Add to your PATH, add this to your .zshrc/.bashrc

export PATH="$HOME/.local/bin:$PATH"


