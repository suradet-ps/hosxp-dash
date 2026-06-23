# Contributing to HOSxp Dash

First of all, thank you for considering contributing to the **HOSxp Dash**! We welcome all contributions, big or small, from developers of all skill levels.

## How to Contribute

There are several ways you can contribute to this project:

- **Reporting Bugs:** If you discover a bug or unexpected behavior, please open an issue. Include detailed steps to reproduce it, along with your operating system and application version.
- **Suggesting Features:** Have an idea for a new feature? We'd love to hear it! Open an issue describing what you want and why it would be useful.
- **Submitting Pull Requests:** Want to write some code? Please feel free to open a Pull Request (PR) adjusting documentation, fixing bugs, or implementing new features.

## Development Workflow

1. **Fork the Repository:** Start by forking the [main repository](https://github.com/suradet-ps/hosxp-dash).
2. **Clone your Fork:**

   ```bash
   git clone https://github.com/suradet-ps/hosxp-dash.git
   cd hosxp-dash
   ```

3. **Create a Branch:** Create a dedicated branch for your work.

   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **Make Changes:** Write your code, update UI, patch bugs, or adjust the Rust backend. Make sure your code is clean and follows the general style of the project.
5. **Test Your Changes:** Ensure the app compiles and your changes work as intended.

   ```bash
   npm run tauri dev
   ```

6. **Commit:** Commit your changes with a clear, descriptive commit message.
7. **Push to your Fork:**

   ```bash
   git push origin feature/your-feature-name
   ```

8. **Open a Pull Request:** Go to the main repository and open a PR matching your feature branch against the `main` branch.

## Coding Guidelines

- **Frontend (Vue/TS):** We use Vue 3 with the Composition API (`<script setup>`). Ensure your components are properly typed using TypeScript.
- **Backend (Rust):** Write safe, idiomatic Rust. Avoid `unwrap()`/`expect()` in favor of proper error handling whenever possible.
- **Styling:** Maintain the professional, modern aesthetic of the application. Please do not introduce large generic styling libraries unless absolutely necessary.
- **Comments:** Follow the human-written, professional codebase comment standards. Avoid noisy, redundant comments.

## Code of Conduct

As contributors and maintainers of this project, we pledge to respect all people who contribute. Please be kind, be respectful of others, and focus on providing constructive feedback.

Thank you for helping us make the HOSxp Dash better for everyone!
