# Planner

A planning app to help me get my shit together, written in rust.

## Motivation

I've been creating a markdown(ish) plan file everyday for several years. I use a naming convention similar to `{yyyy}.{MM}.{dd}.plan.md`, and on most days I create my plan by copying the previous day's plan, then changing the day/date, and amending the schedule.

In my experience, creating my plan by hand each day gives me more agency than allowing a tool to create it for me. However, this is subject to human error, and often results in mismatches between the name of the plan files and the content. It also makes it more difficult to track over time.

## Installation

```console
foo@bar:~$ cargo install planner
```

> To use cargo install, you'll need to [install Rust](https://www.rust-lang.org/tools/install).

## Usage

### Default

```console
foo@bar:~$ planner
```

Opens today's plan in vim. If today's plan doesn't exist, then it will be created by copying the most recent plan. If no plan exists, then an empty plan will be created.

### Help

```sh
foo@bar:~$ planner --help
```

## TODOs

- [ ] Add unit tests for PlanDirectory, PlanFile
- [ ] On plan creation:
  - [ ] Add or modify day/date header
  - [ ] Add or modify TODO section
  - [ ] Add or modify Schedule section
  - [ ] Add or modify notes section
- [ ] Subcommand: `review`: Review most recent plan
  - [ ] Show day and date
  - [ ] Show completed tasks
  - [ ] Show notes
- [ ] Configuration
  - [ ] Change editor preference (not everybody loves vim).
