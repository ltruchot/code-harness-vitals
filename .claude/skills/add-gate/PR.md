# The gate's pull request

Pushing publishes the branch, so confirm with the user first.

```sh
git push -u origin gate/<id>
gh pr create --draft --base main --title "Add the <id> gate" --body-file <scratchpad>/pr-<id>.md
```

The body, in this order:

```markdown
## Summary

<the gate, its pillar, and what it catches for each agent, in two or three lines>

## Probes

<the probe table from ADR 4>

## Sources

<each behavior claim, with its docs URL or OpenCode commit and file>

## Tests

<counts per level (unit, real tree, container), and the `cargo xtask check` result>

## Not verified

<everything that is not proven, and what is not probed yet>
```

End the body with the PR attribution lines from the session instructions.

Run the `code-review` skill on the PR number. Fix each confirmed finding in a new commit
on the branch, never with an amend: the PR history is the review trail.
