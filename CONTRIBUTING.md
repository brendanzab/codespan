# Contributing

## Code of Conduct

Please note that this project is released with a [Code of Conduct](./CODE_OF_CONDUCT.md).
By participating in this project you agree to abide by its terms.

## Matrix Room

Joining the matrix room at [#codespan:matrix.org][codespan-matrix] is a good way to get in touch with the developers and community.

[codespan-matrix]: https://app.element.io/#/room/#codespan:matrix.org

## Contribution Workflow

Follow these steps to contribute to the project:

1. Make a fork of the [codespan repository][codespan-repo].
1. Within your fork, create a branch for your contribution. Use a meaningful name.
1. Create your contribution, meeting all [contribution quality standards](#quality-standards).
1. Ensure all the tests pass (`cargo test`).
1. [Create a pull request][create-a-pr] against the `master` branch of the repository.
1. Once the pull request is reviewed and CI passes, it will be merged.

[codespan-repo]: https://github.com/brendanzab/codespan
[create-a-pr]: https://help.github.com/articles/creating-a-pull-request-from-a-fork/

## Quality Standards

Most quality and style standards are checked automatically by the CI build.
Contributions should:

- Separate each **logical change** into its own commit.
- Include tests for any new functionality and fixed issues in your pull request.
- Update the changelogs with any added, removed, changed, or fixed functionality.
- Document public functions.
- Format code with `cargo fmt`.
- Avoid adding `unsafe` code.
  If it is necessary, provide an explanatory comment on any `unsafe` block explaining its rationale and why it's safe.
- Add a descriptive message for each commit.
  Follow [these commit message guidelines][commit-messages].
- Document your pull requests.
  Include the reasoning behind each change, and the testing done.

[commit-messages]: https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html

## Release Process

1. Bump the version numbers of each crate appropriately.
1. Update the changelogs with the new version ranges.
1. Create a new PR for the release, and if it passes CI merge it.
1. Create a new tag for the release, pointing to the merge commit.
1. Run the following commands in order from the root of the repository.
    Note that doing this too quickly may result in an error,
    due to a server-side delay in crate publishing:
    ```
    cd codespan-reporting && cargo publish; cd ..
    cd codespan && cargo publish; cd ..
    cd codespan-lsp && cargo publish; cd ..
    ```
