## Change

Describe the behavior, documentation, or release metadata change.

## Validation

List the commands you ran.

```sh
make verify
make package-list
make package
```

## Checklist

- [ ] The change is focused on one concern.
- [ ] Public API changes appear in README and docs.
- [ ] Tests are self-contained and use one assertion per test.
- [ ] Raw field names never become SQL identifiers.
- [ ] New dependencies are explained in the pull request.
