# Replay observations — expense-portal (2026-11-21)

Each scenario in `scenarios.json` was replayed with `web_site_journey_run` in Craft Anything's
private browser against the fixture served on port 8768; all seven passed. The steps are the
reference journeys in `../../../benchmark/expectations/2026-11-21-v3.json`.

The two field-error scenarios' errors are announced (`role="alert"`) but no field is marked
invalid and the focus stays on Next, so no gap to a field is measured: on the original they are
not shown next to their field.

`observed_verified` on this synthetic fixture; used without human confirmation by decision of
2026-11-21 (`BENCHMARK_PROTOCOL.md`, addendum 2026-11-21).
