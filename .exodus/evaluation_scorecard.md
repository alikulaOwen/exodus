# Project Exodus Benchmark Scorecard

## Whole-fixture transform tier

**Total Fixtures**: 11 | **Exodus Compile+Outcome Pass Rate**: 54.5% (Baseline: 0.0%)

**Compilation Rate**: 63.6% (Baseline: 100.0%) | **Migration Debts**: 1 | **Human Interventions**: 1

| Fixture | Symbols | Exodus Outcome | Compiled | Debts | Human Review | Baseline Compiled |
|---|---|---|---|---|---|---|
| `01_typed_functions` | 3 | `Verified` | ✅ | 0 | None | ✅ |
| `02_class_conversion` | 1 | `Verified` | ✅ | 0 | None | ✅ |
| `03_module_dependency` | 2 | `Verified` | ❌ | 0 | None | ✅ |
| `04_circular_dependency` | 2 | `Verified` | ❌ | 0 | ⚠️ Required | ✅ |
| `05_unsupported_decorator` | 2 | `Verified` | ❌ | 0 | None | ✅ |
| `06_dynamic_value` | 1 | `Verified` | ✅ | 0 | None | ✅ |
| `07_missing_sdk` | 1 | `Verified` | ❌ | 0 | None | ✅ |
| `08_async_function` | 1 | `Verified` | ✅ | 0 | None | ✅ |
| `09_database_compat_wrapper` | 1 | `Verified` | ✅ | 0 | None | ✅ |
| `10_deliberately_untranslatable_reflection` | 2 | `Degraded` | ✅ | 1 | None | ✅ |
| `two_run_demo` | 2 | `Verified` | ✅ | 0 | None | ✅ |

## Unit-level verification tier (separate from the transform tier above)

**Fixtures with grounded contracts**: 5 | **Units evaluated**: 9

- Verified: 8 | Compatible: 0 | Degraded (compiled, ungrounded): 0 | Blocked: 1
- Unit contract pass rate: 94.1% (16/17 assertions)
- Grounded-oracle coverage: 100.0%
- Repair attempts: 1 | Cases captured: 1
