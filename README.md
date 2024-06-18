# Workshop Repo: Refactoring Rust

*NOTE*: This is Work-In-Progress! Please check for updates a day before the workshop.

This GitHub repo will contain all the examples and workshops files we create during our time together.

## Install Rust

[Rustup](https://rustup.rs) provides you with all the software to compile and run Rust applications, e.g.

1. Cargo - build tool and package manager
2. `rustfmt` - Auto-formatting tool for Rust code
3. `clippy` - Linting for common mistakes

[and many more](https://rust-lang.github.io/rustup-components-history/). *Rustup* also allows you to install different compile targets and multiple toolchains, as well as keeping your toolchains up to date.

After installing, you should have a set of new command line tools available.

### Verify your Rust installation:

1. Open a Terminal/Shell/Command Line of your choice
2. Check out this repo
3. Navigate to this repository
4. Enter

```bash
$ cargo run
```
5. Open your browser at https://localhost:3000

## Recommended Editor

During the workshop, we will use [Visual Studio Code](https://code.visualstudio.com/) as editor. It's free, fast and very extensible. Making yourself familiar with VS Code is highly recommended.

However, working with VS Code is not required. If you have a preferred editor with Rust support you're more productive with, please feel free to use whatever you like. What we highyly recommend though, is checking if your editor has support for [Rust analyzer](https://rust-analyzer.github.io/).

## Recommended VS Code Extensions

To work effeciently, please install a couple of extensions that help you developing Rust. *Note*: Please don't install the recommendend Rust extension. It's outdated and the community decided to move to other tools. You can search and install VS Code extensions through the menu on the side

We recommend the following extensions:

- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer). This is the main extension for Rust development, with the best language support available. *Note*: This extension is also available for other IDEs and editors, check out [their website](https://rust-analyzer.github.io/)

- [crates](https://marketplace.visualstudio.com/items?itemName=serayuzgur.crates). This extension helps installing dependencies from crates.io

- [Better TOML](https://marketplace.visualstudio.com/items?itemName=bungcip.better-toml). TOML is the format that the dependency manager Cargo uses to manage dependencies. This extension helps formatting and editing TOML files

- [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb). All Rust code is compiled against LLVM. This extension helps debugging LLVM code inside VS Code

- [Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens). Inline errors

## Tasks

This tutorial is divided into multiple exercises. You can find each solution in a branch called `step-N` (starting with 0).

Goal: The application in this repository is a simple web server that serves as a Key-Value store. For stored images, it provides additional endpoints to modify the existing images. The `grayscale` endpoint is our main focus. We are going to refactor this piece of code to make it more readable, maintainable and testable.

### Step 0: Syntax

- [ ] Look at the `grayscale` endpoint and try to understand what's going on.
- [ ] Look at the way the developer handled errors, is there a better way to do so? `unwrap`s are allowed for this step.
- [ ] Try splitting up the function into a part to retrieve the data you want to work with and the actual logic to convert the image to grayscale.

### Step 1: Error Handling

- [ ] Let's get rid of all the `unwrap`s in our code.
- [ ] Create a custom error type for our application.
- [ ] Create shortcuts to get proper responses for specific situations
- [ ] Apply the correct traits to make sure we align well with the eco-system
- [ ] Discuss: Why do we need to implement the `Error` trait.

### Step 2: Custom Types

- [ ] Create a custom type for image responses
- [ ] Make sure to align the type with the Axum ecosystem
- [ ] Create proper conversions from the types used in your application and your types
- [ ] Try to name the patterns you've been using
- [ ] Discuss: Do we hide too much? How do we communicate intent?

### Step 3: Polymorphism

- [ ] Create a custom type that represents the stored data
- [ ] Differentiate between images and other thigns
- [ ] Make the type compatible with the rest of the application. Adjust `get_kv`, `post_kv` where necessary
- [ ] Discuss: Is it ok to change our model?
- [ ] Implement a `thumbnail` function and see the 🪄 happen!

### Step 4: Specialization

- [ ] Prepare our application to serve different databases
- [ ] Create a trait that represents the operations on a database
- [ ] Implement the trait for our current database, the HashMap
- [ ] Discuss: ownership in that context
- [ ] Make all your functions generic over the database type
- [ ] Bonus: Implement the trait for a different database
