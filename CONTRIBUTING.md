# Contributing to Islamic Shariah Validator (ISV)

First off, thank you for considering contributing to ISV! 🎉  
Your help is essential for keeping it robust, compliant, and useful for the community.

The following guidelines will help you understand how to report issues, propose enhancements, and submit code changes.

---

## 📌 Code of Conduct

By participating in this project, you agree to abide by our **Code of Conduct**.  
We are committed to providing a welcoming and harassment-free experience for everyone, regardless of gender, sexual orientation, disability, appearance, religion, or technical background.

- Be respectful and inclusive.
- Give constructive feedback.
- Focus on what is best for the community.

Unacceptable behavior will not be tolerated. If you witness or experience any issues, please contact the project maintainer at `abdelhakimzouai@gmail.com`.

---

## 🐛 Reporting Bugs

If you find a bug, please open an issue on [GitLab Issues](https://gitlab.com/ethikluxry/islamic_shariah_validator/-/issues) with the following information:

- **Summary** – a clear and concise description of the problem.
- **Steps to Reproduce** – detailed steps to reproduce the behavior.
- **Expected Behavior** – what you expected to happen.
- **Actual Behavior** – what actually happened.
- **Environment** – Rust version, OS, features enabled, etc.
- **Additional context** – logs, screenshots, or relevant code snippets.

**Before submitting**, please search existing issues to avoid duplicates.

---

## 💡 Suggesting Enhancements

We welcome ideas for new features, improvements, or rule additions.  
Open an issue with the label `enhancement` and describe:

- The current limitation or problem.
- The proposed solution.
- Why this would benefit the project.
- Any potential drawbacks or alternatives.

If your proposal involves **new Shariah rules** or modifications to existing ones, please provide references to AAOIFI standards or scholarly opinions to support it.

---

## 🚀 Pull Request Process

### 1. Fork and Branch

- Fork the repository on GitLab.
- Create a feature branch from `main`:
  ```bash
  git checkout -b feature/your-feature-name
  ```

### 2. Development Setup

Make sure you have Rust **1.70+** installed. Then:

```bash
git clone git@gitlab.com:ethikluxry/islamic_shariah_validator.git
cd islamic_shariah_validator
cargo build --features full
```

### 3. Coding Standards

- **Formatting**: Run `cargo fmt` before committing.
- **Linting**: Ensure `cargo clippy -- -D warnings` passes.
- **Documentation**: Add doc comments (`///`) for public items. Update the `README.md` if needed.
- **Commit messages**: Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:
  - `feat:` new feature
  - `fix:` bug fix
  - `docs:` documentation changes
  - `test:` adding/updating tests
  - `refactor:` code refactoring
  - `chore:` maintenance tasks

### 4. Testing

- Run all tests to ensure nothing breaks:
  ```bash
  cargo test --features full
  ```
- If you add new validation rules, add corresponding **unit tests** in the `tests` module inside the source file or in `tests/` integration tests.

### 5. Commit and Push

- Commit your changes with a clear message:
  ```bash
  git commit -m "feat: add Zakat validation rule"
  ```
- Push to your fork:
  ```bash
  git push origin feature/your-feature-name
  ```

### 6. Create a Merge Request

- Go to the [Merge Requests](https://gitlab.com/ethikluxry/islamic_shariah_validator/-/merge_requests) page and click **New merge request**.
- Select your branch and the `main` branch.
- Provide a descriptive title and explain your changes in detail.
- Link any related issues.

### 7. Review Process

A maintainer will review your MR. Please be responsive to feedback. Once approved, your changes will be merged.

---

## 🧪 Adding or Modifying Shariah Rules

When extending the validator:

1. **Define the rule** – add a new `ViolationCode` variant if needed.
2. **Implement the check** – create a private method in `ShariahValidator`.
3. **Call it** from the `validate()` method.
4. **Add tests** – ensure both positive (valid) and negative (invalid) scenarios are covered.
5. **Update documentation** – mention the new rule in the `README.md` table.

If the rule relies on external data (e.g., asset existence), consider adding an async hook via the `fabric` feature.

---

## 🔧 CI/CD and Automation

This project uses **GitLab CI** to run tests, lints, and build WASM on every push.  
Make sure your branch passes all pipelines before requesting a merge.

To run the same checks locally (optional):

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --features full
cargo build --release --features full
```

---

## 📄 License

By contributing, you agree that your contributions will be licensed under the **DWPL-2.0** license (see [LICENSE](LICENSE)).

---

## 📬 Questions?

If you have any questions or need clarification, reach out via:

- Email: [abdelhakimzouai@gmail.com](mailto:abdelhakimzouai@gmail.com)
- GitLab Issues: [https://gitlab.com/ethikluxry/islamic_shariah_validator/-/issues](https://gitlab.com/ethikluxry/islamic_shariah_validator/-/issues)

---

**Thank you for helping make this project better!** ❤️
```

---

## ✅ Pourquoi ce fichier est "parfait" :

1. **Structure claire** – sections distinctes (bugs, enhancements, PR process, etc.).
2. **Standards modernes** – utilise les commits conventionnels, `cargo fmt`, `clippy`, et tests.
3. **Adapté à la Charia** – mentionne les règles spécifiques et les références AAOIFI.
4. **Code de conduite** – inclus pour garantir un environnement respectueux.
5. **Lien avec la licence** – rappelle que les contributions sont sous DWPL-2.0.

Placez ce fichier à la racine de votre projet et commitez-le. Votre projet est désormais complet avec **README**, **LICENSE**, **CONTRIBUTING** et le code Rust ! 🎉
