# Contributing

## Contents

- [Introduction](#introduction)
- [Code of Conduct](#code-of-conduct)
- [Matrix Room](#matrix-room)
- [Reporting Bugs and Suggesting Improvements](#reporting-bugs-and-suggesting-improvements)
- [Contribution Workflow](#contribution-workflow)
- [Quality Standards](#quality-standards)
- [Release Process](#release-process)

## Introduction

Hello, and welcome to the contributing guide for Codespan!

Codespan is mostly maintained in the spare time of contributors,
so immediate reactions are not to be expected!
By following this guide you'll make it easier for us to address your issues or incorporate your contributions.

We look forward to working with you!

## Code of Conduct

Please note that this project is released with a [Code of Conduct](./CODE_OF_CONDUCT.md).
By participating in this project you agree to abide by its terms.

## Matrix Room

Joining the matrix room at [#codespan:matrix.org][codespan-matrix] is a good way to get in touch with the developers and community.

[codespan-matrix]: https://app.element.io/#/room/#codespan:matrix.org

## Reporting Bugs and Suggesting Improvements

Bugs (unwanted behaviour) and suggested improvements are tracked as [GitHub issues][github-issues].
Before reporting an issue, please check the following points:

1. The issue is caused by `codespan-reporting` itself and not by how it is used.
  Have a look at the documentation if you are not sure.
  If the documentation is not helpful, you can contact the developers at the above matrix chat address or make an issue.
1. Your issue has not already been reported by someone else.
  Please look through the open issues in the [issue tracker][github-issues].

When reporting an issue, please add as much relevant information as possible.
This will help developers and maintainers to resolve your issue. Some things you might consider:

* Use a descriptive title.
* Describe how a problem can be reproduced. Ideally give a minimal example.
* Explain what exactly is the problem and what you expect instead. If it is related to rendering, add screenshots or other illustrations.

[github-issues]: https://github.com/brendanzab/codespan/issues

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
