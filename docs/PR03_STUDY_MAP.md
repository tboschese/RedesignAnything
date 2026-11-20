# PR-03 — Study → capability/coverage mapper

## Tools

- `redesign_study_map` — map Craft `capture/` routes and/or import fixture oracle JSON
- `redesign_capabilities_list` — paginated list
- `redesign_coverage_show` — coverage with `percent_claimed: null` when unknowns remain

## Guarantees tested

- Fixture oracle unknowns stay `unknown` and never yield a 100% claim
- Capture routes become `observed` capabilities with explicit in-page gaps
- No new browser crawler (reuses Craft Study artifacts)

## Example

```bash
craft-anything-cli redesign-study-map \
  --project /path/to/project \
  --oracle-path REDESIGN_SPEC/fixtures/dense-form/oracle/scenarios.json \
  --format json
```
