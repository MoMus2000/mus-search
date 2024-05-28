# Mus-Search

TF-IDF based search using Rust

## Introduction

Index your files and search for keywords in a jiffy.

## Features

- index
- search

PDF, TXT support

API support, MS-Word, Google doc support to come !

## Prerequisites

List the software and tools that need to be installed before setting up your project.

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

## Installation

Step-by-step instructions on how to install and set up your Rust project. Include prerequisites, dependencies, and any other important information.

```sh
# Clone the repository
git clone https://github.com/momus2000/mus-search.git

# Navigate to the project directory
cd mus_search

# Build the project
cargo build

# generate the index
cargo run index

# perform search
cargo run search "Your Query"

```