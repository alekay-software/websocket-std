# How to contribute

The contribution procedure is based on the fork and pull model described [here](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/getting-started/about-collaborative-development-models#fork-and-pull-model).

- Check open issues.
- Fork the project.
- Make your changes.
- Create a pull request.

## Branch naming

TODO

## The pull request should follow this template

The title shoul contain one of the following tags:
- ``ref``: refactor old code.
- ``feat``: new feature.
- ``fix``: fix a bug .
- ``test``: related to the testing part.
- ``doc``: documenting the project.
- ``typo``: fix typos.

The body should contain in the first line the title, then a resume of the changes made and finally the issue that the PR close.

## Example
| Title                      | Body  |
|----------------------------|---|
| feat: SSL for sync client  | feat: SSL for sync client </br></br>Websocket over TLS connections </br></br>Close [Add TLS for sync client #34]|
