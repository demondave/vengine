# Contributing Guidelines

Thank you for contributing! Follow these guidelines to ensure a smooth process.

- [Issue Tracker](https://github.com/colon-crab-colon/game/issues)
- [Kanban Board](https://github.com/orgs/colon-crab-colon/projects/1)

## Reporting Bugs
Check the [Issue Tracker](https://github.com/colon-crab-colon/game/issues).

## Submitting Ideas
Check the [Issue Tracker](https://github.com/colon-crab-colon/game/issues).

## Style Guide

### Coding Conventions
- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) and [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `rustfmt` to format your code:
    ```sh
    cargo fmt
     ```
- Lint your code with `clippy`:
    ```sh
    cargo clippy
    ```

### Commit Messages
We use [Conventional Commits](https://www.conventionalcommits.org/) to maintain consistent commit messages. Follow this format.

## Submitting Changes
1. Create a feature branch:
   ```sh
   git checkout -b feature/your-feature-name
   ```
2. Commit your changes with a semantic commit message:
   ```plaintext
   feat(rendering): improve voxel lighting system
   ```
3. Push your branch:
   ```sh
   git push origin feature/your-feature-name
   ```
4. Open a pull request with a clear description of your changes.

## Running Tests
- Run all tests:
    ```sh
    cargo test
    ```

## Code of Conduct
Follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Thank You
We appreciate your contributions and the time you've dedicated to improving this project!
